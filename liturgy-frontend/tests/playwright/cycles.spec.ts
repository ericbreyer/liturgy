import { test, expect, checkLayout } from './real-data'

test('audited identities align across all five source calendars', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto('/nerd?date=2026-10-07&calendars=54,ef,monastic,of,of-us&scope=year')
  const row = (name: string) => page.locator('.feast-row').filter({
    has: page.getByRole('rowheader', { name, exact: true }),
  })
  await expect(row('St. Cyril of Alexandria').locator('time')).toHaveText([
    '2026-02-09', '2026-02-09', '2026-01-28', '2026-06-27', '2026-06-27',
  ])
  await expect(row('St. Cyril of Jerusalem').locator('time')).toHaveText(Array(5).fill('2026-03-18'))
  await expect(row('St. Cyril of Jerusalem').locator('.status-label')).toHaveText(Array(5).fill('Same date'))
  await expect(row('St. Augustine of Canterbury').locator('time')).toHaveText([
    '2026-05-28', '2026-05-28', '2026-05-26', '2026-05-27', '2026-05-27',
  ])
  await expect(row('St. Augustine of Hippo').locator('time')).toHaveText(Array(5).fill('2026-08-28'))
  const magdalene = row('St. Mary Magdalene')
  await expect(magdalene).toHaveCount(1)
  await expect(magdalene.locator('time')).toHaveText(Array(5).fill('2026-07-22'))
  await expect(magdalene.locator('.status-label')).toHaveText(Array(5).fill('Same date'))
  await expect(magdalene.locator('.titles')).toHaveText(['Penitent', 'Penitent', 'Penitent'])
  await expect(magdalene.locator('.rank')).toHaveText(['Rank: Double', 'Rank: III', 'Rank: III', 'Rank: Feast', 'Rank: Feast'])
  await expect(row("St. Mary Magdalene de' Pazzi").locator('time')).toHaveText(['2026-05-25', '2026-05-25'])
  await expect(row('The Seven Holy Founders of the Servite Order').locator('time')).toHaveText([
    '2026-02-12', '2026-02-12', '2026-02-17', '2026-02-17',
  ])
  await checkLayout(page, testInfo)
})

for (const width of [320, 768, 1280]) {
  test(`source cycle comparisons at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 })
    await page.goto('/nerd?date=2026-10-03&calendars=ef,monastic,of-us')
    await expect(page.locator('.loading')).toHaveCount(0)
    await expect(page.getByRole('radio', { name: 'Temporal', exact: true })).toHaveCount(0)
    await expect(page.getByRole('searchbox', { name: 'Search full cycle' })).toHaveCount(0)
    const therese = page.locator('.feast-row').filter({ hasText: 'Thérèse' })
    await expect(therese).toHaveCount(1)
    await expect(therese.locator('time')).toHaveText(['2026-10-03', '2026-10-03', '2026-10-01'])
    await expect(page.locator('.comparison-results')).not.toContainText(/Sabbato|week XXVI/)
    await expect(therese).not.toContainText('Transferred')
    await checkLayout(page, testInfo)

    await page.getByRole('radio', { name: 'Full cycle', exact: true }).check()
    await expect(therese.locator('.status-label')).toHaveText([
      'Different date', 'Different date', 'Different date',
    ])
    await expect(page.locator('.comparison-results')).not.toContainText('On this date')
    await expect(page.locator('.feast-row')).not.toHaveCount(1)
    await expect(page.locator('.feast-row').filter({ hasText: 'Francis of Assisi' })).toBeVisible()
    const search = page.getByRole('searchbox', { name: 'Search full cycle' })
    await search.fill('therese')
    await expect(page.locator('.feast-row')).toHaveCount(1)
    await expect(therese.locator('time')).toHaveText(['2026-10-03', '2026-10-03', '2026-10-01'])
    await checkLayout(page, testInfo)
    await search.fill('no such saint')
    await expect(page.getByText('No matching feasts.', { exact: true })).toBeVisible()
    await search.fill('francis assisi')
    await expect(page.locator('.feast-row')).toHaveCount(1)
    await expect(page.locator('.feast-row')).toContainText('Francis of Assisi')
    await search.fill('')
    await expect(page.locator('.feast-row')).not.toHaveCount(1)
    await expect(page.locator('.loading')).toHaveCount(0)
    await checkLayout(page, testInfo)
  })
}

for (const width of [320, 1280]) {
  test(`October 7 distinguishes the Marks and compares full-cycle dates at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 })
    await page.goto('/nerd?date=2026-10-07&calendars=ef,monastic,of-us')
    const pope = page.locator('.feast-row').filter({
      has: page.getByRole('rowheader', { name: 'St. Mark, Pope and Confessor', exact: true }),
    })
    await expect(pope).toHaveCount(1)
    await expect(pope.locator('time')).toHaveText(['2026-10-07'])
    await expect(pope.locator('.status-label')).toHaveText(['On this date', 'Not in cycle', 'Not in cycle'])
    await expect(page.locator('.comparison-results')).not.toContainText('Evangelist')
    const rosary = page.locator('.feast-row').filter({
      has: page.getByRole('rowheader', { name: 'Blessed Virgin Mary of the Rosary', exact: true }),
    })
    await expect(rosary.locator('time')).toHaveText(['2026-10-07', '2026-10-07', '2026-10-07'])
    await checkLayout(page, testInfo)

    await page.getByRole('radio', { name: 'Full cycle', exact: true }).check()
    const evangelist = page.locator('.feast-row').filter({
      has: page.getByRole('rowheader', { name: 'St. Mark, Evangelist', exact: true }),
    })
    await expect(evangelist.locator('time')).toHaveText(['2026-04-25', '2026-04-25', '2026-04-25'])
    await expect(evangelist.locator('.status-label')).toHaveText(['Same date', 'Same date', 'Same date'])
    await expect(pope.locator('.status-label')).toHaveText(['Only in this calendar', 'Not in cycle', 'Not in cycle'])
    await expect(rosary.locator('.status-label')).toHaveText(['Same date', 'Same date', 'Same date'])
    await page.locator('.date-picker').fill('2026-10-08')
    await expect(evangelist.locator('.status-label')).toHaveText(['Same date', 'Same date', 'Same date'])
    await expect(page.locator('.comparison-results')).not.toContainText('On this date')
    await checkLayout(page, testInfo)
  })
}