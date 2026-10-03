import { describe, it, expect, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createHead } from '@vueuse/head'
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
          [{
            desc: 'Commemoration A',
            rank: 'Memorial',
            date: '2025-09-13',
            color: 'white',
          }, 'Commemoration'],
          [{
            desc: 'Commemoration B',
            rank: 'Optional',
            date: '2025-09-13',
            color: 'blue',
          }, 'Commemoration'],
        ],
      },
    }),
  },
}))

describe('TodayView snapshot', () => {
  it('renders consistent DOM structure', async () => {
    const wrapper = mount(TodayView, { global: { plugins: [createHead()] } })
    try {
      await flushPromises()
      expect(wrapper.text()).toContain('Test Feast')
      expect(wrapper.text()).toContain('Commemoration A')
      expect(wrapper.text()).toContain('Commemoration B')
      expect(wrapper.html()).toMatchSnapshot()
    } finally {
      wrapper.unmount()
    }
  })
})
