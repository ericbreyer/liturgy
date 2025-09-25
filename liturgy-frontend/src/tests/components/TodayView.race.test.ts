import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'

vi.useFakeTimers()

describe('TodayView race conditions', () => {
  let selectedDateRef: any = ref('2025-09-13')

  beforeEach(() => {
    selectedDateRef.value = '2025-09-13'

    // Use runtime mocks so they can close over selectedDateRef
    vi.doMock('../../composables/useCalendarSelection', () => ({
      useCalendarSelection: () => ({
        selectedCalendars: { value: ['default'] },
        loadCalendars: () => Promise.resolve(),
        selectedCalendarInfos: { value: [{ name: 'default', commemoration_interpretation: 'Commemorations' }] },
      }),
    }))

    vi.doMock('../../composables/useDateNavigation', () => ({
      useDateNavigation: () => ({
        selectedDate: selectedDateRef,
        formattedDate: { value: new Date('2025-09-13').toDateString() },
        updateSelectedDate: () => {},
        goToToday: () => {},
        goToPrevious: () => {},
        goToNext: () => {},
        route: { query: {} },
      }),
    }))

    vi.doMock('../../services/api', () => ({
      api: {
        getDayInfo: vi.fn(async (calendar: string, year: number, month: number, day: number, signal?: AbortSignal) => {
          // Day 13 -> slow (200ms), Day 14 -> fast (50ms)
          const delay = day === 13 ? 200 : 50
          return await new Promise((resolve, reject) => {
            const t = setTimeout(() => {
              resolve({
                desc: {
                  date: `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`,
                  day_in_season: 'Season',
                  day_rank: 'Feast',
                  day: { desc: `Feast ${day}`, rank: 'Feast', date: '', color: 'green' },
                  commemorations: [],
                },
              })
            }, delay)

            // If signal aborts, cancel timer and reject with AbortError
            if (signal) {
              signal.addEventListener('abort', () => {
                clearTimeout(t)
                const err: any = new Error('Aborted')
                err.name = 'AbortError'
                reject(err)
              })
            }
          })
        }),
      },
    }))
  })

  afterEach(() => {
    vi.resetAllMocks()
  })

  it('applies only latest response when switching dates quickly', async () => {
    // Import inside test to ensure mocks applied
    const TodayView = (await import('../../views/TodayView.vue')).default
    const wrapper = mount(TodayView, { attachTo: document.body })

    // Let mount trigger the first load (for day 13)
    await Promise.resolve()

    // Immediately switch to day 14
    selectedDateRef.value = '2025-09-14'

    // Advance timers so the fast (day 14) resolves first
    vi.advanceTimersByTime(60)
    await Promise.resolve()

    // Now advance more so the slow one would have resolved if not aborted
    vi.advanceTimersByTime(200)
    await Promise.resolve()

    // Check that the DOM contains 'Feast 14' and not 'Feast 13'
    const html = wrapper.html()
    expect(html).toContain('Feast 14')
    expect(html).not.toContain('Feast 13')
  })
})
