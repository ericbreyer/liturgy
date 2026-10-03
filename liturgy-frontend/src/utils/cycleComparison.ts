import type { CycleFeast } from '../services/api'
import { feastIdentity } from './feastIdentity'
export { normalizeFeastName } from './feastIdentity'

export type CycleScope = 'day' | 'year'
export type CycleSort = 'date' | 'name'
export type CycleStatus = 'On this date' | 'Different date' | 'Same date' | 'Only in this calendar' | 'Not in cycle' | 'No date this year'

export interface CycleComparison {
  identity: string
  name: string
  firstDate: string | null
  calendars: Record<string, CycleFeast[]>
}

export function cycleStatus(
  feast: CycleFeast | undefined,
  selectedDate: string,
  scope: CycleScope = 'day',
  comparison?: CycleComparison,
  calendar?: string,
): CycleStatus {
  if (!feast) return 'Not in cycle'
  if (!feast.date) return 'No date this year'
  if (scope === 'year' && comparison) {
    const otherCalendars = Object.entries(comparison.calendars)
      .filter(([name]) => name !== calendar)
    if (!otherCalendars.length) return 'Only in this calendar'
    return otherCalendars.every(([, feasts]) => feasts.some((other) => other.date === feast.date))
      ? 'Same date'
      : 'Different date'
  }
  return feast.date === selectedDate ? 'On this date' : 'Different date'
}

export function compareCycles(
  cycles: Record<string, CycleFeast[]>,
  selectedDate: string,
  scope: CycleScope,
  sort: CycleSort = 'date',
): CycleComparison[] {
  const comparisons = new Map<string, CycleComparison>()
  for (const [calendar, feasts] of Object.entries(cycles)) {
    for (const feast of feasts) {
      const { key: identity, name } = feastIdentity(feast)
      if (!identity) continue
      let comparison = comparisons.get(identity)
      if (!comparison) {
        comparison = { identity, name, firstDate: null, calendars: {} }
        comparisons.set(identity, comparison)
      }
      const calendarFeasts = comparison.calendars[calendar] ?? []
      calendarFeasts.push(feast)
      comparison.calendars[calendar] = calendarFeasts
      if (feast.date && (!comparison.firstDate || feast.date < comparison.firstDate)) {
        comparison.firstDate = feast.date
      }
    }
  }
  const names = new Map<string, number>()
  for (const comparison of comparisons.values()) {
    names.set(comparison.name, (names.get(comparison.name) ?? 0) + 1)
  }
  for (const comparison of comparisons.values()) {
    if ((names.get(comparison.name) ?? 0) > 1) {
      const feast = Object.values(comparison.calendars)[0]?.[0]
      if (feast) comparison.name = feast.description
    }
  }
  return [...comparisons.values()]
    .filter(
      (comparison) =>
        scope === 'year' ||
        Object.values(comparison.calendars).some((feasts) =>
          feasts.some((feast) => feast.date === selectedDate),
        ),
    )
    .sort((left, right) => {
      const byName =
        left.name.localeCompare(right.name) || left.identity.localeCompare(right.identity)
      return sort === 'name'
        ? byName
        : (left.firstDate ?? '9999-99-99').localeCompare(right.firstDate ?? '9999-99-99') || byName
    })
}
