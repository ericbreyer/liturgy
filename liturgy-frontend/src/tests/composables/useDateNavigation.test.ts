import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useDateNavigation } from '../../composables/useDateNavigation'

// Mock Vue Router
const mockPush = vi.fn()
const mockRoute = {
  query: { date: '2024-12-25' } as Record<string, string | string[] | undefined>
}

vi.mock('vue-router', () => ({
  useRoute: () => mockRoute,
  useRouter: () => ({
    push: mockPush
  })
}))

describe('useDateNavigation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockRoute.query = { date: '2024-12-25' }
  })

  it('should return selected date from query param', () => {
    const { selectedDate } = useDateNavigation('Today')
    
    expect(selectedDate.value).toBe('2024-12-25')
  })

  it('should return today if no date in query', () => {
    mockRoute.query = {}
    const { selectedDate } = useDateNavigation('Today')
    
    // Should be today's date
    const today = new Date().toISOString().split('T')[0]
    expect(selectedDate.value).toBe(today)
  })

  it('should format date for display (Today view)', () => {
    const { formattedDate } = useDateNavigation('Today')
    
    // Should include weekday, month, day, year
    expect(formattedDate.value).toMatch(/Wednesday, December 25, 2024/)
  })

  it('should format date for display (Month view)', () => {
    const { formattedDate } = useDateNavigation('Month')
    
    // Should only include month and year
    expect(formattedDate.value).toBe('December 2024')
  })

  it('should update selected date', () => {
    const { updateSelectedDate } = useDateNavigation('Today')
    
    updateSelectedDate('2025-01-01')
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Today',
      query: { date: '2025-01-01' }
    })
  })

  it('should go to today', () => {
    const { goToToday } = useDateNavigation('Week')
    
    goToToday()
    
    const today = new Date().toISOString().split('T')[0]
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Week',
      query: { date: today }
    })
  })

  it('should go to previous day (Today view)', () => {
    const { goToPrevious } = useDateNavigation('Today')
    
    goToPrevious()
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Today',
      query: { date: '2024-12-24' }
    })
  })

  it('should go to next day (Today view)', () => {
    const { goToNext } = useDateNavigation('Today')
    
    goToNext()
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Today',
      query: { date: '2024-12-26' }
    })
  })

  it('should go to previous week (Week view)', () => {
    const { goToPrevious } = useDateNavigation('Week')
    
    goToPrevious()
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Week',
      query: { date: '2024-12-18' }
    })
  })

  it('should go to next week (Week view)', () => {
    const { goToNext } = useDateNavigation('Week')
    
    goToNext()
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Week',
      query: { date: '2025-01-01' }
    })
  })

  it('should go to previous month (Month view)', () => {
    const { goToPrevious } = useDateNavigation('Month')
    
    goToPrevious()
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Month',
      query: { date: '2024-11-25' }
    })
  })

  it('should go to next month (Month view)', () => {
    const { goToNext } = useDateNavigation('Month')
    
    goToNext()
    
    expect(mockPush).toHaveBeenCalledWith({
      name: 'Month',
      query: { date: '2025-01-25' }
    })
  })

  it('should handle invalid date in query', () => {
    mockRoute.query = { date: 'invalid-date' }
    const { selectedDate } = useDateNavigation('Today')
    
    // Should fallback to today
    const today = new Date().toISOString().split('T')[0]
    expect(selectedDate.value).toBe(today)
  })

  it('should return route object', () => {
    const { route } = useDateNavigation('Today')
    
    expect(route).toBe(mockRoute)
  })
})
