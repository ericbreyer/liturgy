import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { ref } from 'vue'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import NerdView from '../../views/NerdView.vue'
import { api, type CycleFeast } from '../../services/api'

const selectedCalendars = ref(['ef', 'monastic', 'of-us'])
vi.mock('../../composables/useCalendarSelection', () => ({
  useCalendarSelection: () => ({ selectedCalendars }),
}))
vi.mock('../../services/api', () => ({
  api: { getCalendarCycle: vi.fn(), getDayInfo: vi.fn(), searchFeasts: vi.fn() },
}))

function feast(name = 'StTherese', date: string | null = '2026-10-03'): CycleFeast {
  return {
    name,
    description: 'Saint Therese',
    date,
    date_rule: '10-03',
    rank: 'III',
    color: 'White',
    titles: ['Virgin'],
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: Error) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

describe('NerdView source cycles', () => {
  let wrapper: ReturnType<typeof mount<typeof NerdView>> | undefined
  let router: Router

  async function open(query: Record<string, string> = {}) {
    router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/nerd', name: 'Nerd', component: NerdView }],
    })
    await router.push({
      name: 'Nerd',
      query: { date: '2026-10-03', calendars: 'ef,monastic,of-us', ...query },
    })
    await router.isReady()
    wrapper = mount(NerdView, { global: { plugins: [router] } })
    await flushPromises()
    return wrapper
  }

  beforeEach(() => {
    vi.resetAllMocks()
    selectedCalendars.value = ['ef', 'monastic', 'of-us']
    vi.mocked(api.getCalendarCycle).mockImplementation(async (calendar) => [
      feast('StTherese', calendar === 'of-us' ? '2026-10-01' : '2026-10-03'),
    ])
  })
  afterEach(() => {
    wrapper?.unmount()
    wrapper = undefined
  })

  it('loads only source definitions and shows StTherese prescribed dates, not observed ferias', async () => {
    const view = await open()
    expect(api.getCalendarCycle).toHaveBeenCalledTimes(3)
    expect(api.getCalendarCycle).toHaveBeenCalledWith(
      'ef',
      'sanctoral',
      2026,
      expect.any(AbortSignal),
    )
    expect(view.findAll('.feast-row')).toHaveLength(1)
    expect(view.findAll('time').map((date) => date.text())).toEqual([
      '2026-10-03',
      '2026-10-03',
      '2026-10-01',
    ])
    expect(view.text()).toContain('Different date')
    expect(view.text()).not.toMatch(/Sabbato|feria|Transferred|Rank changed/)
    expect(api.getDayInfo).not.toHaveBeenCalled()
    expect(api.searchFeasts).not.toHaveBeenCalled()
  })

  it('filters dates and scope locally, preserves query, and caches calendar reselection', async () => {
    const view = await open()
    await router.push({ query: { ...router.currentRoute.value.query, date: '2026-10-04' } })
    await flushPromises()
    expect(view.text()).toContain('No source feasts')
    await view.get('input[value="year"]').setValue()
    await flushPromises()
    expect(router.currentRoute.value.query).toMatchObject({
      scope: 'year',
      date: '2026-10-04',
      calendars: 'ef,monastic,of-us',
    })
    expect(view.findAll('.feast-row')).toHaveLength(1)
    expect(view.findAll('.status-label').map((label) => label.text())).toEqual([
      'Different date', 'Different date', 'Different date',
    ])
    expect(view.text()).not.toContain('On this date')
    selectedCalendars.value = []
    await flushPromises()
    expect(view.findAll('.feast-row')).toHaveLength(0)
    expect(view.text()).toContain('Select at least one calendar')
    selectedCalendars.value = ['ef']
    await flushPromises()
    expect(view.findAll('.feast-row')).toHaveLength(1)
    expect(view.find('.status-label').text()).toBe('Only in this calendar')
    expect(api.getCalendarCycle).toHaveBeenCalledTimes(3)
  })

  it('ignores legacy temporal links and reloads sanctoral data for a new year', async () => {
    const view = await open({ cycle: 'temporal' })
    expect(view.find('input[value="temporal"]').exists()).toBe(false)
    expect(api.getCalendarCycle).toHaveBeenCalledTimes(3)
    expect(vi.mocked(api.getCalendarCycle).mock.calls.every((call) => call[1] === 'sanctoral')).toBe(true)
    await router.push({ query: { ...router.currentRoute.value.query, date: '2027-10-03' } })
    await flushPromises()
    expect(api.getCalendarCycle).toHaveBeenCalledWith(
      'ef',
      'sanctoral',
      2027,
      expect.any(AbortSignal),
    )
  })

  it('renders the full cycle union, undated definitions, missing cells and source metadata', async () => {
    vi.mocked(api.getCalendarCycle).mockImplementation(async (calendar) =>
      calendar === 'ef'
        ? [feast('Zulu', null), feast('Alpha', '2026-12-01'), feast('Beta', '2026-01-01')]
        : [],
    )
    const view = await open({ cycle: 'sanctoral', scope: 'year' })
    expect(view.findAll('.feast-name').map((cell) => cell.text())).toEqual([
      'Beta',
      'Alpha',
      'Zulu',
    ])
    expect(view.text()).toContain('No date this year')
    expect(view.text()).toContain('Not in cycle')
    expect(view.text()).toContain('Rule: 10-03')
    expect(view.text()).toContain('Virgin')
    expect(view.text()).toContain('Rank: III')
    expect(view.text()).not.toContain('Rank (')
    await view.get('select[aria-label="Sort cycle"]').setValue('name')
    expect(view.findAll('.feast-name').map((cell) => cell.text())).toEqual([
      'Alpha',
      'Beta',
      'Zulu',
    ])
  })

  it('searches full-cycle names, source descriptions, titles and dates locally', async () => {
    vi.mocked(api.getCalendarCycle).mockResolvedValue([
      { ...feast('St. Thérèse', '2026-10-03'), description: 'Saint Therese of Lisieux', titles: ['Virgin', 'Doctor of the Church'] },
      { ...feast('St. Francis', '2026-10-04'), description: 'Saint Francis of Assisi', titles: ['Confessor'] },
    ])
    const view = await open({ scope: 'year' })
    const input = view.get('input[type="search"]')
    for (const query of ['THERESE', 'lisieux', 'doctor church', '2026-10-03']) {
      await input.setValue(query)
      expect(view.findAll('.feast-row')).toHaveLength(1)
      expect(view.find('.feast-name').text()).toBe('St. Thérèse')
    }
    await input.setValue('no such saint')
    expect(view.findAll('.feast-row')).toHaveLength(0)
    expect(view.text()).toContain('No matching feasts.')
    await input.setValue('')
    expect(view.findAll('.feast-row')).toHaveLength(2)
    await input.setValue('francis')
    await view.get('input[value="day"]').setValue()
    await flushPromises()
    expect(view.find('input[type="search"]').exists()).toBe(false)
    expect(view.find('.feast-name').text()).toBe('St. Thérèse')
    await view.get('input[value="year"]').setValue()
    await flushPromises()
    expect(view.find('.feast-name').text()).toBe('St. Francis')
    expect(api.getCalendarCycle).toHaveBeenCalledTimes(3)
  })

  it('shows empty cycles, errors without false absence, and recovers on reselection', async () => {
    vi.mocked(api.getCalendarCycle).mockResolvedValue([])
    const view = await open()
    expect(view.text()).toContain('No source feasts')
    vi.mocked(api.getCalendarCycle).mockImplementation(async (calendar) => {
      if (calendar === 'of-us') throw new Error('Cycle unavailable')
      return [feast('StTherese', '2027-10-03')]
    })
    await router.push({ query: { ...router.currentRoute.value.query, date: '2027-10-03' } })
    await flushPromises()
    expect(view.get('[role="alert"]').text()).toContain('OF-US: Cycle unavailable')
    expect(view.findAll('.feast-cell')[2]!.text()).toBe('Unavailable')
    selectedCalendars.value = ['ef']
    await flushPromises()
    vi.mocked(api.getCalendarCycle).mockResolvedValue([feast('StTherese', '2027-10-03')])
    selectedCalendars.value = ['ef', 'of-us']
    await flushPromises()
    expect(view.find('[role="alert"]').exists()).toBe(false)
    expect(view.find('.loading').exists()).toBe(false)
  })

  it('rejects stale success/error/finally writes and aborts on clearing selection or unmount', async () => {
    selectedCalendars.value = ['ef']
    const old = deferred<CycleFeast[]>()
    const current = deferred<CycleFeast[]>()
    vi.mocked(api.getCalendarCycle)
      .mockReturnValueOnce(old.promise)
      .mockReturnValueOnce(current.promise)
    const view = await open()
    const oldSignal = vi.mocked(api.getCalendarCycle).mock.calls[0]![3]!
    await router.push({ query: { ...router.currentRoute.value.query, date: '2027-10-03' } })
    await flushPromises()
    expect(oldSignal.aborted).toBe(true)
    old.resolve([feast('Stale')])
    await flushPromises()
    expect(view.text()).not.toContain('Stale')
    expect(view.find('.loading').exists()).toBe(true)
    current.resolve([feast('Current', '2027-10-03')])
    await flushPromises()
    expect(view.text()).toContain('Current')
    const pending = deferred<CycleFeast[]>()
    vi.mocked(api.getCalendarCycle).mockReturnValueOnce(pending.promise)
    await router.push({ query: { ...router.currentRoute.value.query, date: '2028-10-03' } })
    await flushPromises()
    const pendingSignal = vi.mocked(api.getCalendarCycle).mock.calls.at(-1)![3]!
    selectedCalendars.value = []
    await flushPromises()
    expect(pendingSignal.aborted).toBe(true)
    pending.reject(new Error('Stale error'))
    await flushPromises()
    expect(view.find('.error').exists()).toBe(false)
    expect(view.find('.loading').exists()).toBe(false)
    const unmounted = deferred<CycleFeast[]>()
    vi.mocked(api.getCalendarCycle).mockReturnValueOnce(unmounted.promise)
    selectedCalendars.value = ['ef']
    await flushPromises()
    const lastSignal = vi.mocked(api.getCalendarCycle).mock.calls.at(-1)![3]!
    view.unmount()
    wrapper = undefined
    expect(lastSignal.aborted).toBe(true)
    unmounted.resolve([])
    await flushPromises()
  })
})
