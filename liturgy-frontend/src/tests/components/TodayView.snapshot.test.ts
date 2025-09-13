import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import TodayView from '../../views/TodayView.vue'

// Mock composables and services used by TodayView
vi.mock('../../composables/useCalendarSelection', () => ({
  useCalendarSelection: () => ({
    selectedCalendars: { value: ['default'] },
    loadCalendars: () => Promise.resolve(),
    selectedCalendarInfos: {
      value: [{ name: 'default', commemoration_interpretation: 'Commemorations' }],
    },
  }),
}))

vi.mock('../../composables/useDateNavigation', () => ({
  useDateNavigation: () => ({
    selectedDate: { value: new Date().toISOString().split('T')[0] },
    formattedDate: { value: new Date().toDateString() },
    updateSelectedDate: () => {},
    goToToday: () => {},
    goToPrevious: () => {},
    goToNext: () => {},
    route: { query: {} },
  }),
}))

vi.mock('../../services/api', () => ({
  api: {
    getDayInfo: async () => ({
      desc: {
        date: new Date().toISOString().split('T')[0],
        day_in_season: 'Season Day 123',
        day_rank: 'Feast',
        day: {
          desc: 'Test Feast',
          rank: 'Feast',
          date: new Date().toISOString().split('T')[0],
          color: 'green',
        },
        commemorations: [
          {
            desc: 'Commemoration A',
            rank: 'Memorial',
            date: new Date().toISOString().split('T')[0],
            color: 'white',
          },
          {
            desc: 'Commemoration B',
            rank: 'Optional',
            date: new Date().toISOString().split('T')[0],
            color: 'blue',
          },
        ],
      },
    }),
  },
}))

describe('TodayView snapshot', () => {
  it('renders consistent DOM structure', async () => {
    const wrapper = mount(TodayView, { attachTo: document.body })
    // Wait a tick for async mounted hooks
    await new Promise((r) => setTimeout(r, 0))
    expect(wrapper.html()).toMatchSnapshot()
  })
})
