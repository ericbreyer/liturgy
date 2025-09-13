/**
 * Liturgical utility functions for feast days, colors, and date handling
 */

/**
 * Convert liturgical color names to CSS color values
 */
export function getColorValue(color: string): string {
  const colorMap: Record<string, string> = {
    white: '#ffffff',
    red: '#dc2626',
    green: '#16a34a',
    violet: '#7c3aed',
    purple: '#7c3aed',
    rose: '#f43f5e',
    gold: '#eab308',
    yellow: '#eab308',
    black: '#1f2937',
  }
  return colorMap[color?.toLowerCase()] || '#6b7280'
}

/**
 * Check if a feast description indicates a feria (ordinary weekday) or optional BVM commemoration
 */
export function isFeria(description: string): boolean {
  const feriaPatterns = [
    /^(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday) of/i,
    /^(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday) in/i,
    /^Weekday/i,
    /^Feria/i,
    /^Sabato/i,
    /^Dominica/i,
    /^(First|Second|Third|Fourth|Fifth|Sixth|Seventh) Day of/i,
    /^Day \d+ of/i,
    /^Ordinary Time/i,
  ]

  // BVM commemorations on Saturdays/Sundays (optional memorials)
  const bvmPatterns = [
    /^(Blessed Virgin Mary|BVM)/i,
    /^Our Lady/i,
    /^Mary/i,
    /^The Blessed Virgin/i,
  ]

  // Check for ferias first
  if (feriaPatterns.some((pattern) => pattern.test(description.trim()))) {
    return true
  }

  // Check for BVM commemorations (these are typically optional on Saturdays)
  if (bvmPatterns.some((pattern) => pattern.test(description.trim()))) {
    // Additional check: if it contains common BVM Saturday titles
    const saturdayBvmPatterns = [/Saturday/i, /Memorial of.*Mary/i, /of the Blessed Virgin Mary$/i]
    if (saturdayBvmPatterns.some((pattern) => pattern.test(description.trim()))) {
      return true
    }
  }

  return false
}

/**
 * Get status icon for feast comparison
 */
export function getStatusIcon(status: string): string {
  const icons: Record<string, string> = {
    present: '✓',
    absent: '✗',
    transferred: '↪',
    'rank-changed': '△',
    'found-elsewhere': '📍',
  }
  return icons[status] || '?'
}

/**
 * Get human-readable status label for feast comparison
 */
export function getStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    present: 'Present',
    absent: 'Not observed',
    transferred: 'Transferred',
    'rank-changed': 'Different rank',
    'found-elsewhere': 'Found on different date',
  }
  return labels[status] || 'Unknown'
}

/**
 * Format a date for display
 */
export function formatLiturgicalDate(date: Date): string {
  return date.toLocaleDateString('en-US', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}

/**
 * Get liturgical rank priority (higher number = higher rank)
 */
export function getRankPriority(rank: string): number {
  const rankMap: Record<string, number> = {
    solemnity: 100,
    feast: 80,
    memorial: 60,
    'optional memorial': 40,
    commemoration: 20,
    feria: 10,
  }
  return rankMap[rank?.toLowerCase()] || 0
}

/**
 * Get rank value for feast comparison (used in calendar sorting)
 */
export function getRankValue(rank: string): number {
  const rankOrder: Record<string, number> = {
    'Greater Feast': 100,
    'Principal Feast': 90,
    Feast: 80,
    'Lesser Feast': 70,
    Commemoration: 60,
    Optional: 50,
    Weekday: 10,
  }
  return rankOrder[rank] || 0
}

/**
 * Convert calendar code to readable name for consistent display
 */
export function getCalendarName(calendar: string): string {
  return calendar.toUpperCase()
}
