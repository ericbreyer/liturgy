import { describe, it, expect } from 'vitest'
import {
  getColorValue,
  isFeria,
  getStatusIcon,
  getStatusLabel,
  getRankPriority,
  getRankValue,
  getCalendarName,
} from '../../utils/liturgical'

describe('liturgical utils', () => {
  describe('getColorValue', () => {
    it('should return correct color for white', () => {
      expect(getColorValue('white')).toBe('#ffffff')
    })

    it('should return correct color for red', () => {
      expect(getColorValue('red')).toBe('#dc2626')
    })

    it('should return correct color for green', () => {
      expect(getColorValue('green')).toBe('#16a34a')
    })

    it('should return correct color for violet', () => {
      expect(getColorValue('violet')).toBe('#7c3aed')
    })

    it('should handle purple as violet', () => {
      expect(getColorValue('purple')).toBe('#7c3aed')
    })

    it('should handle case insensitive input', () => {
      expect(getColorValue('RED')).toBe('#dc2626')
      expect(getColorValue('White')).toBe('#ffffff')
    })

    it('should return default color for unknown input', () => {
      expect(getColorValue('unknown')).toBe('#6b7280')
    })

    it('should handle null/undefined input', () => {
      expect(getColorValue('')).toBe('#6b7280')
      expect(getColorValue(null as any)).toBe('#6b7280')
    })
  })

  describe('isFeria', () => {
    it('should identify weekday ferias', () => {
      expect(isFeria('Monday of the 1st Week')).toBe(true)
      expect(isFeria('Tuesday in Ordinary Time')).toBe(true)
      expect(isFeria('Wednesday of the 2nd Week')).toBe(true)
      expect(isFeria('Weekday')).toBe(true)
      expect(isFeria('Feria')).toBe(true)
    })

    it('should identify feria patterns', () => {
      expect(isFeria('Thursday of the 3rd Week of Advent')).toBe(true)
      expect(isFeria('Friday in the Octave of Christmas')).toBe(true)
      expect(isFeria('Saturday of the 4th Week of Lent')).toBe(true)
      expect(isFeria('Ordinary Time')).toBe(true)
    })

    it('should identify BVM commemorations only with Saturday context', () => {
      // These should NOT be ferias without Saturday context
      expect(isFeria('Blessed Virgin Mary')).toBe(false)
      expect(isFeria('Our Lady of Sorrows')).toBe(false)
      expect(isFeria('Mary of the Incarnation')).toBe(false)

      // These SHOULD be ferias with Saturday/Memorial context (starting with BVM patterns)
      expect(isFeria('BVM on Saturday')).toBe(true)
      expect(isFeria('Blessed Virgin Mary on Saturday')).toBe(true)
      expect(isFeria('Our Lady Memorial on Saturday')).toBe(true)
      expect(isFeria('Mary Memorial of the Blessed Virgin Mary')).toBe(true)
    })

    it('should not identify proper feasts as ferias', () => {
      expect(isFeria('Christmas Day')).toBe(false)
      expect(isFeria('Easter Sunday')).toBe(false)
      expect(isFeria('Saint Peter')).toBe(false)
      expect(isFeria('Nativity of Saint John the Baptist')).toBe(false)
    })

    it('should handle case insensitive input', () => {
      expect(isFeria('MONDAY OF THE 1ST WEEK')).toBe(true)
      expect(isFeria('tuesday in ordinary time')).toBe(true)
      // BVM without Saturday context should be false
      expect(isFeria('blessed virgin mary')).toBe(false)
    })
  })

  describe('getRankPriority', () => {
    it('should return correct priority for solemnity', () => {
      expect(getRankPriority('solemnity')).toBe(100)
    })

    it('should return correct priority for feast', () => {
      expect(getRankPriority('feast')).toBe(80)
    })

    it('should return correct priority for memorial', () => {
      expect(getRankPriority('memorial')).toBe(60)
    })

    it('should handle case insensitive input', () => {
      expect(getRankPriority('SOLEMNITY')).toBe(100)
      expect(getRankPriority('Feast')).toBe(80)
    })

    it('should return 0 for unknown ranks', () => {
      expect(getRankPriority('unknown')).toBe(0)
      expect(getRankPriority('')).toBe(0)
    })
  })

  describe('getRankValue', () => {
    it('should return correct value for Greater Feast', () => {
      expect(getRankValue('Greater Feast')).toBe(100)
    })

    it('should return correct value for Principal Feast', () => {
      expect(getRankValue('Principal Feast')).toBe(90)
    })

    it('should return correct value for Feast', () => {
      expect(getRankValue('Feast')).toBe(80)
    })

    it('should return correct value for Lesser Feast', () => {
      expect(getRankValue('Lesser Feast')).toBe(70)
    })

    it('should return 0 for unknown ranks', () => {
      expect(getRankValue('Unknown Rank')).toBe(0)
      expect(getRankValue('')).toBe(0)
    })
  })

  describe('getCalendarName', () => {
    it('should convert calendar code to uppercase', () => {
      expect(getCalendarName('of-us')).toBe('OF-US')
      expect(getCalendarName('ef')).toBe('EF')
    })

    it('should handle already uppercase input', () => {
      expect(getCalendarName('OF-US')).toBe('OF-US')
    })

    it('should handle mixed case input', () => {
      expect(getCalendarName('Of-Us')).toBe('OF-US')
    })

    it('should handle empty string', () => {
      expect(getCalendarName('')).toBe('')
    })
  })
})
