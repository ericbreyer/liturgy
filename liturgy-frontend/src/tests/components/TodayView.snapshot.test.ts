import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import TodayView from '../../views/TodayView.vue'

// Mock composables and services used by TodayView
vi.mock('../../composables/useCalendarSelection', () => {
  const { ref } = require('vue')
  return {
    useCalendarSelection: () => ({
      selectedCalendars: ref(['default']),
      loadCalendars: () => Promise.resolve(),
      selectedCalendarInfos: ref([{ name: 'default', commemoration_interpretation: 'Commemorations' }]),
    }),
  }
})

vi.mock('../../composables/useDateNavigation', () => {
  const { ref } = require('vue')
  // Use a fixed date so snapshot is stable
  const fixedDate = '2025-09-13'
  return {
    useDateNavigation: () => ({
      selectedDate: ref(fixedDate),
      formattedDate: ref(new Date(fixedDate).toDateString()),
      updateSelectedDate: () => {},
      goToToday: () => {},
      goToPrevious: () => {},
      goToNext: () => {},
      route: { query: {} },
    }),
  }
})

vi.mock('../../services/api', () => ({
  api: {
    getDayInfo: async () => ({
      desc: {
        date: '2025-09-13',
        day_in_season: 'Season Day 123',
        day_rank: 'Feast',
        day: {
          desc: 'Test Feast',
          rank: 'Feast',
          date: '2025-09-13',
          color: 'green',
        },
        commemorations: [
          {
            desc: 'Commemoration A',
            rank: 'Memorial',
            date: '2025-09-13',
            color: 'white',
          },
          {
            desc: 'Commemoration B',
            rank: 'Optional',
            date: '2025-09-13',
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
