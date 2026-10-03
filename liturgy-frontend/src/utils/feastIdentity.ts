import type { CycleFeast } from '../services/api'

export function normalizeFeastName(name: string): string {
  return name
    .normalize('NFD')
    .replace(/\p{M}/gu, '')
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]/gu, '')
}

const aliasGroups = [
  ['Our Lady of the Rosary', 'Blessed Virgin Mary of the Rosary', 'The most Holy Rosary of the Blessed Virgin Mary'],
  ['Our Lady of Mount Carmel', 'Blessed Virgin Mary of Mt. Carmel', 'Commemoration of the Blessed Virgin Mary of Mt. Carmel'],
  ['The Most Sacred Heart of Jesus', 'Sacred Heart of Jesus'],
  ['The Seven Holy Founders of the Servite Order', 'The Seven Holy Founders of the Order of Servants of the Blessed Virgin Mary'],
  ['The Nativity of the Blessed Virgin Mary', 'Nativity of the Blessed Virgin Mary'],
  ['The Visitation of the Blessed Virgin Mary', 'Visitation of the Blessed Virgin Mary'],
  ['The Queenship of the Blessed Virgin Mary', 'Queenship of the Blessed Virgin Mary'],
  ['The Immaculate Heart of the Blessed Virgin Mary', 'Immaculate Heart of the Blessed Virgin Mary'],
  ['The Conversion of St. Paul the Apostle', 'Conversion of St. Paul'],
  ['Ss. Mark and Marcellianus', 'Ss. Mark and Marcellian'],
  ['The Nativity of St. John the Baptist', 'Nativity of St. John the Baptist'],
  ['Vigil of the Nativity of St. John the Baptist', 'Vigil of St. John the Baptist'],
  ['Seven Sorrows of the Blessed Virgin Mary', 'Commemoration of the Seven Sorrows of the Blessed Virgin Mary'],
  ['The Commemoration of St. Paul', 'Commemoration of St. Paul'],
  ['The Beheading of St. John the Baptist', 'Beheading of St. John the Baptist', 'The Passion of St. John the Baptist'],
  ['The Dedication of the Basilicas of Ss. Peter and Paul', 'Dedication of the Basilicas of Ss. Peter and Paul'],
  ['The Chair of St. Peter the Apostle', 'Chair of St. Peter', 'Chair of St. Peter at Antioch'],
  ['Our Lady of Lourdes', 'Apparition of the Blessed Virgin Mary', 'The Apparition of the Blessed Virgin Mary Immaculate'],
  ['The Baptism of the Lord', 'Commemoration of the Baptism of our Lord Jesus Christ'],
  ['St. Paulinus of Nola', 'St. Paulinus'],
]

const aliases = new Map(aliasGroups.flatMap(([canonical, ...names]) =>
  [canonical!, ...names].map((name) => [normalizeFeastName(name), normalizeFeastName(canonical!)] as const),
))

function titleKey(titles: string[]): string {
  return [...new Set(titles.map(normalizeFeastName))].sort().join('|')
}

function sourceKey(name: string, titles: string[]): string {
  const normalized = normalizeFeastName(name)
  return `${aliases.get(normalized) ?? normalized}:${titleKey(titles)}`
}

type SourceIdentity = [name: string, titles: string[]]
type IdentityGroup = [canonical: SourceIdentity, ...variants: SourceIdentity[]]

const identityGroups: IdentityGroup[] = [
  [['St. Mary Magdalene', []],
    ['St. Mary Magdalene', ['Penitent']]],
  [['St. Cyril of Alexandria', ['Bishop', 'Doctor of the Church']],
    ['St. Cyril', ['Bishop of Alexandria', 'Confessor', 'Doctor of the Church']]],
  [['St. Cyril of Jerusalem', ['Bishop', 'Doctor of the Church']],
    ['St. Cyril', ['Bishop of Jerusalem', 'Confessor', 'Doctor of the Church']],
    ['St. Cyril Bishop of Jerusalem, Confessor, and', ['Doctor of the Church']]],
  [['St. Augustine of Canterbury', ['Bishop']],
    ['St. Augustine', ['Bishop', 'Confessor']]],
  [['St. Augustine of Hippo', ['Bishop', 'Doctor of the Church']],
    ['St. Augustine', ['Bishop', 'Confessor', 'Doctor of the Church']],
    ['St. Augustine Bishop, Confessor, and', ['Doctor of the Church']]],
  [['St. Ignatius of Antioch', ['Bishop', 'Martyr']],
    ['St. Ignatius', ['Bishop', 'Martyr']]],
  [['St. Ignatius of Loyola', ['Priest']], ['St. Ignatius', ['Confessor']]],
  [['St. Margaret of Scotland', []], ['St. Margaret', ['Queen', 'Widow']]],
  [['St. Stephen of Hungary', []], ['St. Stephen', ['King', 'Confessor']]],
  [['St. Stephen I', ['Pope', 'Martyr']], ['St. Stephen', ['Pope', 'Martyr']]],
  [['St. Eusebius of Vercelli', ['Bishop']], ['St. Eusebius', ['Bishop', 'Martyr']]],
  [['St. Thomas Becket', ['Bishop', 'Martyr']], ['St. Thomas', ['Bishop', 'Martyr']]],
  [['St. Thomas Aquinas', ['Priest', 'Doctor of the Church']],
    ['St. Thomas Aquinas', ['Confessor', 'Doctor of the Church']]],
  [['St. Paulinus of Nola', ['Bishop']], ['St. Paulinus', ['Bishop', 'Confessor']]],
  [['The Seven Holy Founders of the Servite Order', []],
    ['The Seven Holy Founders of the Order of Servants of the Blessed Virgin Mary', ['Confessors']]],
  [['The Conversion of St. Paul the Apostle', []], ['Conversion of St. Paul', ['Apostle']]],
  [['The Chair of St. Peter the Apostle', []], ['Chair of St. Peter', ['Apostle']]],
  [['The Beheading of St. John the Baptist', []], ['The Passion of St. John the Baptist', ['Martyr']]],
  [['The Dedication of the Basilicas of Ss. Peter and Paul', []],
    ['Dedication of the Basilicas of Ss. Peter and Paul', ['Apostles']]],
]

const identities = new Map(identityGroups.flatMap(([canonical, ...variants]) =>
  [canonical, ...variants].map(([name, titles]) => [sourceKey(name, titles), {
    key: sourceKey(...canonical), name: canonical[0],
  }] as const),
))

export function feastIdentity(feast: CycleFeast): { key: string; name: string } {
  const key = sourceKey(feast.name, feast.titles)
  const identity = identities.get(key) ?? { key, name: feast.name }
  const normalized = normalizeFeastName(feast.name)
  const titles = titleKey(feast.titles)
  if ((normalized === 'stpeter' && titles === 'apostle') ||
    (normalized === 'stpaul' && (titles === 'apostle' || !titles))) {
    return { key: `${identity.key}:${feast.date_rule}`, name: identity.name }
  }
  return identity
}