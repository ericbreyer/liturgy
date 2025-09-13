import { describe, it, expect } from 'vitest'
import { addDays, daysBetween, formatDate, getCurrentDate } from '../../utils/dateUtils'

describe('dateUtils', () => {
  describe('addDays', () => {
    it('should add positive days correctly', () => {
      const result = addDays('2025-01-01', 5)
      expect(result).toBe('2025-01-06')
    })

    it('should subtract days with negative input', () => {
      const result = addDays('2025-01-10', -5)
      expect(result).toBe('2025-01-05')
    })

    it('should handle month boundaries', () => {
      const result = addDays('2025-01-31', 1)
      expect(result).toBe('2025-02-01')
    })

    it('should handle year boundaries', () => {
      const result = addDays('2024-12-31', 1)
      expect(result).toBe('2025-01-01')
    })

    it('should handle leap years', () => {
      const result = addDays('2024-02-28', 1)
      expect(result).toBe('2024-02-29')
    })

    it('should handle zero days', () => {
      const result = addDays('2025-06-15', 0)
      expect(result).toBe('2025-06-15')
    })
  })

  describe('daysBetween', () => {
    it('should calculate positive difference', () => {
      const result = daysBetween('2025-01-01', '2025-01-10')
      expect(result).toBe(9)
    })

    it('should calculate negative difference', () => {
      const result = daysBetween('2025-01-10', '2025-01-01')
      expect(result).toBe(-9)
    })

    it('should return 0 for same date', () => {
      const result = daysBetween('2025-01-01', '2025-01-01')
      expect(result).toBe(0)
    })

    it('should handle month boundaries', () => {
      const result = daysBetween('2025-01-31', '2025-02-01')
      expect(result).toBe(1)
    })

    it('should handle year boundaries', () => {
      const result = daysBetween('2024-12-31', '2025-01-01')
      expect(result).toBe(1)
    })
  })

  describe('formatDate', () => {
    it('should format date correctly', () => {
      const result = formatDate('2025-01-01')
      expect(result).toMatch(/Wednesday, January 1, 2025/)
    })

    it('should handle different months', () => {
      const result = formatDate('2025-06-15')
      expect(result).toMatch(/Sunday, June 15, 2025/)
    })

    it('should handle leap year dates', () => {
      const result = formatDate('2024-02-29')
      expect(result).toMatch(/Thursday, February 29, 2024/)
    })
  })

  describe('getCurrentDate', () => {
    it('should return date in YYYY-MM-DD format', () => {
      const result = getCurrentDate()
      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/)
    })

    it('should return current date', () => {
      const result = getCurrentDate()
      // Check format is correct (YYYY-MM-DD)
      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/)

      // Check it's a valid date string
      const resultDate = new Date(result)
      expect(resultDate).toBeInstanceOf(Date)
      expect(resultDate.toString()).not.toBe('Invalid Date')

      // Check it's reasonably close to now (within a day)
      const now = new Date()
      const diffMs = Math.abs(now.getTime() - resultDate.getTime())
      const diffDays = diffMs / (1000 * 60 * 60 * 24)
      expect(diffDays).toBeLessThan(2) // Should be within 2 days due to timezone differences
    })
  })
})
