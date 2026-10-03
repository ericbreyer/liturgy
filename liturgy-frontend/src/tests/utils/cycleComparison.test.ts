import { afterEach, describe, expect, it, vi } from 'vitest'
import { api, type CycleFeast } from '../../services/api'
import { compareCycles, cycleStatus, normalizeFeastName } from '../../utils/cycleComparison'

function feast(name: string, date: string | null, extra: Partial<CycleFeast> = {}): CycleFeast {
  return {
    name,
    date,
    description: name,
    date_rule: '10-03',
    rank: 'III',
    color: 'White',
    titles: [],
    ...extra,
  }
}

describe('source cycle comparison', () => {
  it('keeps Mark the Pope separate from Mark the Evangelist on October 7', () => {
    const evangelist = feast('St. Mark', '2026-04-25', {
      titles: ['Evangelist'], description: 'St. Mark, Evangelist',
    })
    const pope = feast('St. Mark', '2026-10-07', {
      titles: ['Pope', 'Confessor'], description: 'St. Mark, Pope and Confessor',
    })
    const cycles = { ef: [evangelist, pope], monastic: [evangelist], of: [evangelist] }
    const dayRows = compareCycles(cycles, '2026-10-07', 'day')
    expect(dayRows).toHaveLength(1)
    expect(dayRows[0]!.name).toBe('St. Mark, Pope and Confessor')
    expect(dayRows[0]!.calendars).toEqual({ ef: [pope] })
    expect(compareCycles(cycles, '2026-10-07', 'year')).toHaveLength(2)
  })

  it('matches the explicit Rosary names across calendars', () => {
    const rows = compareCycles({
      ef: [feast('Blessed Virgin Mary of the Rosary', '2026-10-07')],
      of: [feast('Our Lady of the Rosary', '2026-10-07')],
    }, '2026-10-07', 'day')
    expect(rows).toHaveLength(1)
    expect(Object.keys(rows[0]!.calendars)).toEqual(['ef', 'of'])
  })

  it('compares full-cycle dates against other calendars, not the date picker', () => {
    const cycles = {
      ef: [feast('Same', '2026-04-25'), feast('Changed', '2026-10-03'), feast('Unique', '2026-10-07')],
      of: [feast('Same', '2026-04-25'), feast('Changed', '2026-10-01')],
    }
    const rows = compareCycles(cycles, '2026-10-07', 'year')
    for (const selectedDate of ['2026-10-07', '2026-04-25']) {
      const statuses = rows.map((row) => cycleStatus(row.calendars.ef![0], selectedDate, 'year', row, 'ef'))
      expect(statuses).toEqual(['Same date', 'Different date', 'Only in this calendar'])
    }
  })

  it('matches normalized names and titles across prescribed dates, independently of ranks', () => {
    const rows = compareCycles(
      {
        ef: [feast('StThérèse', '2026-10-03', { titles: ['Virgin'] })],
        monastic: [feast('ST-THERESE', '2026-10-03', { titles: ['Virgin'] })],
        'of-us': [feast('StTherese', '2026-10-01', { rank: 'Memorial', titles: ['Virgin'] })],
      },
      '2026-10-03',
      'day',
    )
    expect(rows).toHaveLength(1)
    expect(rows[0]!.calendars['of-us']![0]!.date).toBe('2026-10-01')
    expect(cycleStatus(rows[0]!.calendars['of-us']![0], '2026-10-03')).toBe('Different date')
    expect(normalizeFeastName('St. Thérèse')).toBe('sttherese')
  })

  it('retains suppressed source definitions without injecting observed ferias', () => {
    const rows = compareCycles({ ef: [feast('StTherese', '2026-10-03')] }, '2026-10-03', 'day')
    expect(rows.map((row) => row.name)).toEqual(['StTherese'])
    expect(JSON.stringify(rows)).not.toMatch(/Sabbato|feria/i)
  })

  it('does not fuzzy-match different saints or mistake a title for a name', () => {
    const rows = compareCycles(
      {
        ef: [feast('StTherese', '2026-10-03', { titles: ['Virgin'] })],
        of: [feast('StTeresa', '2026-10-03'), feast('Virgin', '2026-10-03')],
      },
      '2026-10-03',
      'day',
    )
    expect(rows).toHaveLength(3)
    expect(rows.find((row) => row.name === 'StTherese')?.calendars.of).toBeUndefined()
    expect(cycleStatus(undefined, '2026-10-03')).toBe('Not in cycle')
  })

  it('keeps undated and duplicate definitions in the year union and sorts by date or name', () => {
    const cycles = {
      ef: [
        feast('Zulu', null),
        feast('Alpha', '2026-10-03'),
        feast('Alpha', '2026-10-04'),
        feast('Beta', '2026-01-01'),
      ],
    }
    const rows = compareCycles(cycles, '2026-10-03', 'year')
    expect(rows.map((row) => row.name)).toEqual(['Beta', 'Alpha', 'Zulu'])
    expect(rows[1]!.calendars.ef).toHaveLength(2)
    expect(cycleStatus(rows[2]!.calendars.ef![0], '2026-10-03')).toBe('No date this year')
    expect(compareCycles(cycles, '2026-10-03', 'year', 'name').map((row) => row.name)).toEqual([
      'Alpha',
      'Beta',
      'Zulu',
    ])
    expect(compareCycles(cycles, '2026-10-05', 'day')).toEqual([])
  })
})

describe('source cycle API contract', () => {
  afterEach(() => vi.restoreAllMocks())

  it('unwraps the envelope and passes the encoded calendar, cycle, year and signal', async () => {
    const entries = [feast('StTherese', '2026-10-03')]
    const fetchMock = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValue({ json: async () => ({ success: true, data: entries }) } as Response)
    const controller = new AbortController()
    expect(await api.getCalendarCycle('of us', 'sanctoral', 2026, controller.signal)).toEqual(
      entries,
    )
    expect(fetchMock).toHaveBeenCalledWith('/api/calendars/of%20us/cycles/sanctoral/2026', {
      signal: controller.signal,
    })
  })

  it('reports failed envelopes and preserves abort semantics', async () => {
    const fetchMock = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValue({
        json: async () => ({ success: false, error: 'Unavailable' }),
      } as Response)
    await expect(api.getCalendarCycle('ef', 'temporal', 2026)).rejects.toThrow('Unavailable')
    const controller = new AbortController()
    controller.abort()
    fetchMock.mockRejectedValue(new DOMException('Aborted', 'AbortError'))
    await expect(
      api.getCalendarCycle('ef', 'sanctoral', 2026, controller.signal),
    ).rejects.toMatchObject({ name: 'AbortError' })
  })
})
