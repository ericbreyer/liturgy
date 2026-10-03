import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import { createHead } from '@vueuse/head'

describe('TodayView race conditions', () => {
  const selectedDateRef = ref('2025-09-13')
  let wrapper: ReturnType<typeof mount> | undefined

  beforeEach(() => {
    vi.useFakeTimers()
    vi.resetModules()
    selectedDateRef.value = '2025-09-13'

    // Use runtime mocks so they can close over selectedDateRef
    vi.doMock('../../composables/useCalendarSelection', () => ({
      useCalendarSelection: () => ({
        selectedCalendars: ref(['default']),
        loadCalendars: () => Promise.resolve(),
        selectedCalendarInfos: ref([{ name: 'default', commemoration_interpretation: 'Commemorations' }]),
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
                const err = new Error('Aborted')
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
    wrapper?.unmount()
    wrapper = undefined
    vi.clearAllTimers()
    vi.useRealTimers()
    vi.resetAllMocks()
  })

  it('applies only latest response when switching dates quickly', async () => {
    // Import inside test to ensure mocks applied
    const TodayView = (await import('../../views/TodayView.vue')).default
    wrapper = mount(TodayView, { global: { plugins: [createHead()] } })

    // Let mount trigger the first load (for day 13)
    await Promise.resolve()

    // Immediately switch to day 14
    selectedDateRef.value = '2025-09-14'
    await nextTick()

    // Advance timers so the fast (day 14) resolves first
    await vi.advanceTimersByTimeAsync(60)
    await nextTick()
    expect(wrapper.text()).toContain('Feast 14')
    expect(wrapper.text()).not.toContain('Feast 13')

    // Now advance more so the slow one would have resolved if not aborted
    await vi.advanceTimersByTimeAsync(200)
    await nextTick()

    // Check that the DOM contains 'Feast 14' and not 'Feast 13'
    const html = wrapper.html()
    expect(html).toContain('Feast 14')
    expect(html).not.toContain('Feast 13')
  })
})
