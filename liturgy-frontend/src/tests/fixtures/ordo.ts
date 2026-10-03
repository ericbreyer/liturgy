import type { CalendarInfo, Vespers } from '../../services/api'

export const calendars: CalendarInfo[] = [
  {
    name: 'ef',
    display_name: 'Extraordinary Form',
    description: 'Traditional calendar',
    commemoration_interpretation: 'Commemorations',
  },
  {
    name: 'of',
    display_name: 'Ordinary Form',
    description: 'Modern calendar',
    commemoration_interpretation: 'Memorials',
  },
]

export function createVespers(): Vespers {
  return {
    name: 'Test Vespers',
    ordo: {
      antiphons: { type: 'Common', name: 'Confessor' },
      psalms: { type: 'Psalter' },
      chapter: { type: 'Proper' },
      hymn: { type: 'Ordinary', source: 'Advent' },
      verse: { type: 'Sunday', source: 'Advent' },
      magnificat_antiphon: { type: 'Octave', source: 'Christmas' },
      collect: { type: 'Proper' },
    },
    commemorations: [],
  }
}