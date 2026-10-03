import { describe, expect, it } from 'vitest'
import type { CycleFeast } from '../../services/api'
import { feastIdentity } from '../../utils/feastIdentity'
import { compareCycles } from '../../utils/cycleComparison'

function feast(name: string, titles: string[] = [], date_rule = '1/1'): CycleFeast {
  return { name, titles, date_rule, description: [name, ...titles].join(', '),
    date: '2026-01-01', rank: 'III', color: 'White' }
}

describe('audited feast identities', () => {
  it('compares Mary Magdalene title variants together without merging Mary Magdalene de Pazzi', () => {
    const older = { ...feast('St. Mary Magdalene', ['Penitent'], '7/22'), date: '2026-07-22' }
    const penitent = { ...feast('St. Mary Magdalene', ['Penitent'], '7/22'), date: '2026-07-22' }
    const modern = { ...feast('St. Mary Magdalene', [], '7/22'), date: '2026-07-22', rank: 'Feast' }
    const rows = compareCycles({ '54': [older], ef: [penitent], monastic: [penitent], of: [modern], 'of-us': [modern] }, '2026-07-22', 'year')
    expect(rows).toHaveLength(1)
    expect(rows[0]!.name).toBe('St. Mary Magdalene')
    expect(Object.keys(rows[0]!.calendars)).toHaveLength(5)
    expect(rows[0]!.calendars.ef![0]!.titles).toEqual(['Penitent'])
    expect(feastIdentity(modern).key).not.toBe(feastIdentity(feast("St. Mary Magdalene de' Pazzi", ['Virgin'], '5/25')).key)
  })

  it.each([
    ['St. Mark', ['Pope', 'Confessor'], ['Evangelist']],
    ['St. Paul', ['First Hermit', 'Confessor'], ['Apostle']],
    ['St. Agnes', ['Virgin', 'Martyr'], ['Virgin', 'Martyr', 'second']],
    ['St. Timothy', ['Bishop', 'Martyr'], ['Martyr']],
    ['St. Peter', ['Apostle'], ['Martyr']],
    ['St. Ignatius', ['Bishop', 'Martyr'], ['Confessor']],
    ['St. Cyril', ['Bishop of Alexandria', 'Confessor', 'Doctor of the Church'], ['Bishop of Jerusalem', 'Confessor', 'Doctor of the Church']],
    ['St. Boniface', ['Bishop', 'Martyr'], ['Martyr']],
    ['St. Augustine', ['Bishop', 'Confessor'], ['Bishop', 'Confessor', 'Doctor of the Church']],
    ['St. Margaret', ['Queen', 'Widow'], ['Virgin', 'Martyr']],
    ['St. Stephen', ['King', 'Confessor'], ['Protomartyr']],
    ['St. Eusebius', ['Bishop', 'Martyr'], ['Confessor']],
    ['St. Thomas', ['Apostle'], ['Bishop', 'Martyr']],
  ] as [string, string[], string[]][])('separates the reused name %s', (name, firstTitles, secondTitles) => {
    const first = feast(name, firstTitles)
    const second = feast(name, secondTitles)
    expect(feastIdentity(first).key).not.toBe(feastIdentity(second).key)
    expect(compareCycles({ ef: [first, second] }, '2026-01-01', 'year')).toHaveLength(2)
    expect(feastIdentity(first).key).toBe(compareCycles({ ef: [first] }, '2026-01-01', 'year')[0]!.identity)
  })

  it.each([
    ['St. Cyril', ['Bishop of Alexandria', 'Confessor', 'Doctor of the Church'], 'St. Cyril of Alexandria'],
    ['St. Cyril', ['Bishop of Jerusalem', 'Confessor', 'Doctor of the Church'], 'St. Cyril of Jerusalem'],
    ['St. Augustine', ['Bishop', 'Confessor'], 'St. Augustine of Canterbury'],
    ['St. Augustine', ['Bishop', 'Confessor', 'Doctor of the Church'], 'St. Augustine of Hippo'],
    ['St. Ignatius', ['Bishop', 'Martyr'], 'St. Ignatius of Antioch'],
    ['St. Ignatius', ['Confessor'], 'St. Ignatius of Loyola'],
    ['St. Margaret', ['Queen', 'Widow'], 'St. Margaret of Scotland'],
    ['St. Stephen', ['King', 'Confessor'], 'St. Stephen of Hungary'],
    ['St. Stephen', ['Pope', 'Martyr'], 'St. Stephen I'],
    ['St. Eusebius', ['Bishop', 'Martyr'], 'St. Eusebius of Vercelli'],
    ['St. Thomas', ['Bishop', 'Martyr'], 'St. Thomas Becket'],
  ] as [string, string[], string][])('links qualified %s to its full name', (generic, titles, named) => {
    const modernTitles: Record<string, string[]> = {
      'St. Cyril of Alexandria': ['Bishop', 'Doctor of the Church'],
      'St. Cyril of Jerusalem': ['Bishop', 'Doctor of the Church'],
      'St. Augustine of Canterbury': ['Bishop'],
      'St. Augustine of Hippo': ['Bishop', 'Doctor of the Church'],
      'St. Ignatius of Antioch': ['Bishop', 'Martyr'],
      'St. Ignatius of Loyola': ['Priest'],
      'St. Stephen I': ['Pope', 'Martyr'],
      'St. Eusebius of Vercelli': ['Bishop'],
      'St. Thomas Becket': ['Bishop', 'Martyr'],
    }
    expect(feastIdentity(feast(generic, titles)).key).toBe(feastIdentity(feast(named, modernTitles[named] ?? [])).key)
  })

  it('includes every title in ordinary identities, with order and case normalized', () => {
    const bishop = feast('St. Example', ['Bishop', 'Confessor'])
    expect(feastIdentity(bishop).key).toBe(feastIdentity(feast('ST EXAMPLE', ['confessor', 'BISHOP'])).key)
    expect(feastIdentity(bishop).key).not.toBe(feastIdentity(feast('St. Example', ['Bishop', 'Martyr'])).key)
    expect(feastIdentity(bishop).key).not.toBe(feastIdentity(feast('St. Example', ['Bishop'])).key)
    expect(feastIdentity(bishop).key).not.toBe(feastIdentity(feast('St. Example')).key)
    expect(feastIdentity(feast('St. Augustine of Hippo', ['Bishop'])).key)
      .not.toBe(feastIdentity(feast('St. Augustine of Hippo', ['Bishop', 'Doctor of the Church'])).key)
  })

  it.each([
    ['The most Holy Rosary of the Blessed Virgin Mary', 'Our Lady of the Rosary'],
    ['Commemoration of the Blessed Virgin Mary of Mt. Carmel', 'Our Lady of Mount Carmel'],
    ['Sacred Heart of Jesus', ' The Most Sacred Heart of Jesus'],
    ['The Seven Holy Founders of the Order of Servants of the Blessed Virgin Mary', 'The Seven Holy Founders of the Servite Order'],
    ['Visitation of the Blessed Virgin Mary', 'The Visitation of the Blessed Virgin Mary'],
    ['Nativity of the Blessed Virgin Mary', 'The Nativity of the Blessed Virgin Mary'],
    ['Conversion of St. Paul', 'The Conversion of St. Paul the Apostle'],
    ['Ss. Mark and Marcellian', 'Ss. Mark and Marcellianus'],
    ['Chair of St. Peter at Antioch', 'The Chair of St. Peter the Apostle'],
    ['Apparition of the Blessed Virgin Mary', 'Our Lady of Lourdes'],
    ['Beheading of St. John the Baptist', 'The Passion of St. John the Baptist'],
    ['Commemoration of the Baptism of our Lord Jesus Christ', 'The Baptism of the Lord'],
  ])('links confirmed alias %s', (first, second) => {
    expect(feastIdentity(feast(first)).key).toBe(feastIdentity(feast(second)).key)
  })

  it('keeps distinct celebrations and different saint rosters separate', () => {
    for (const [first, second] of [
      ['Chair of St. Peter Apostle at Rome', 'Chair of St. Peter at Antioch'],
      ['Seven Sorrows of the Blessed Virgin Mary', 'Seven Sorrows of the Blessed Virgin Mary (Our Lady of Sorrows)'],
      ['The Nativity of St. John the Baptist', 'Vigil of the Nativity of St. John the Baptist'],
      ['Ss. Marcellinus, Peter, and Erasmus', 'Ss. Marcellinus and Peter'],
    ]) {
      expect(feastIdentity(feast(first!)).key).not.toBe(feastIdentity(feast(second!)).key)
    }
    expect(feastIdentity(feast('St. Peter', ['Apostle'], '1/25')).key)
      .not.toBe(feastIdentity(feast('St. Peter', ['Apostle'], '6/30')).key)
    expect(feastIdentity(feast('St. Paul', ['Apostle'], '1/18')).key)
      .not.toBe(feastIdentity(feast('St. Paul', [], '2/22')).key)
  })

  it('matches changed source dates and ranks without inventing aliases', () => {
    const first = feast('St. Thomas', ['Apostle'], '12/21')
    const second = { ...feast('St. Thomas', ['Apostle'], '7/3'), rank: 'Feast' }
    expect(feastIdentity(first).key).toBe(feastIdentity(second).key)
    expect(feastIdentity(feast('St. Teresa')).key).not.toBe(feastIdentity(feast('St. Therese')).key)
  })
})