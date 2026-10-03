import { test, expect, routes, routeURL, waitForContent, checkLayout, checkBounds, attachScreenshot } from './real-data'

for (const timezoneId of ['America/Los_Angeles', 'Asia/Tokyo']) {
  test.describe(`Date-only rendering in ${timezoneId}`, () => {
    test.use({ timezoneId })

    test('Month dates and feast details agree', async ({ page }) => {
      await page.goto(routeURL('month'))
      await waitForContent(page, 'month')
      const christmas = page.locator('.calendar-day.selected')
      await expect(christmas.locator('.day-number')).toHaveText('25 Thursday')
      await christmas.click()
      await expect(page.locator('.detail-header h3')).toHaveText('Thursday, December 25, 2025')
      await expect(page.locator('.detail-panel')).toContainText('Nativity')
    })
  })
}

const viewports = [320, 360, 390, 480, 640, 768, 769, 1024, 1280, 1440, 1920]
  .map(width => ({ width, height: 900 }))
viewports.push({ width: 568, height: 320 }, { width: 1024, height: 600 })

for (const viewport of viewports) {
  test(`all routes at ${viewport.width}x${viewport.height}`, async ({ page }, testInfo) => {
    test.setTimeout(120_000)
    await page.setViewportSize(viewport)
    for (const route of routes) {
      await test.step(`/${route}: real content and responsive bounds`, async () => {
        try {
          await page.goto(routeURL(route))
          await waitForContent(page, route)
          if (route === 'search') {
            await page.locator('.search-input').fill('Christmas')
            await page.locator('.search-input').press('Enter')
            await expect(page.locator('.result-card').first()).toBeVisible()
            await expect(page.locator('.loading')).toHaveCount(0)
            await expect(page.locator('.calendar-column')).toHaveCount(5)
          }
          await checkLayout(page, testInfo)
          const dropdown = page.locator('.dropdown-toggle')
          if (await dropdown.count()) {
            await dropdown.click()
            await expect(dropdown).toHaveAttribute('aria-expanded', 'true')
            await expect(page.locator('.dropdown-content input[type=checkbox]')).toHaveCount(5)
            await checkBounds(page, page.locator('.dropdown-content'), 'calendar-dropdown', testInfo)
            await checkBounds(page, page.locator('.checkbox-text'), 'calendar-labels', testInfo, { unclipped: true })
            await dropdown.click()
          }
        } catch (error) {
          await attachScreenshot(page, testInfo, `${route}-content-failure`)
          await checkLayout(page, testInfo)
          expect.soft(false, `/${route} at ${viewport.width}x${viewport.height}: ${String(error)}`).toBe(true)
        }
      })
    }
  })
}

test('mobile menu links visit all eight real views', async ({ page }, testInfo) => {
  test.setTimeout(90_000)
  await page.setViewportSize({ width: 360, height: 800 })
  await page.goto(routeURL('today'))
  await waitForContent(page, 'today')
  const menu = page.getByRole('button', { name: 'Toggle navigation' })
  for (const route of routes) {
    await menu.click()
    await expect(menu).toHaveAttribute('aria-expanded', 'true')
    const link = page.locator(`.nav-links a[href^="/${route}"]`)
    await expect(link).toBeVisible()
    await checkBounds(page, link, `${route}-menu-link`, testInfo)
    await link.click()
    await expect(page).toHaveURL(new RegExp(`/${route}(?:\\?|$)`))
    await expect(menu).toHaveAttribute('aria-expanded', 'false')
    await waitForContent(page, route)
  }
})

for (const viewport of [{ width: 320, height: 800 }, { width: 1280, height: 800 }]) {
  test(`calendar empty state and date navigation at ${viewport.width}`, async ({ page }, testInfo) => {
    await page.setViewportSize(viewport)
    await page.goto(routeURL('today'))
    await waitForContent(page, 'today')
    await page.locator('.dropdown-toggle').click()
    await page.getByRole('button', { name: 'Select None', exact: true }).click()
    await expect(page.locator('.no-selection:visible')).toContainText('Please select at least one calendar')
    await expect(page.locator('.dropdown-toggle')).toContainText('0 of 5')
    await page.getByRole('button', { name: 'Select All', exact: true }).click()
    await page.locator('.dropdown-toggle').click()
    await waitForContent(page, 'today')
    await page.getByTitle('Next Day', { exact: true }).click()
    await expect(page).toHaveURL(/date=2025-12-26/)
    await waitForContent(page, 'today')
    await page.getByTitle('Previous Day', { exact: true }).click()
    await expect(page).toHaveURL(/date=2025-12-25/)
    const input = page.getByLabel('Select date', { exact: true })
    await input.fill('2025-12-24')
    await input.press('Tab')
    await expect(page).toHaveURL(/date=2025-12-24/)
    await waitForContent(page, 'today')
    await checkLayout(page, testInfo)
    await page.getByTitle('Go to Today', { exact: true }).click()
    const today = await page.evaluate(() => {
      const date = new Date()
      return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
    })
    await expect(page).toHaveURL(new RegExp(`date=${today}`))
    await waitForContent(page, 'today')
  })
}

for (const viewport of [{ width: 568, height: 320 }, { width: 1024, height: 600 }]) {
  test(`Month detail opens and closes in ${viewport.width}x${viewport.height}`, async ({ page }, testInfo) => {
    await page.setViewportSize(viewport)
    await page.goto(routeURL('month'))
    await waitForContent(page, 'month')
    await page.locator('.calendar-day.selected').click()
    await expect(page.locator('.detail-panel')).toBeVisible()
    await expect(page.locator('.detail-header h3')).toHaveText('Thursday, December 25, 2025')
    await expect(page.locator('.calendar-detail')).toHaveCount(5)
    await checkBounds(page, page.locator('.detail-panel'), 'detail-panel', testInfo, { vertical: true })
    await checkBounds(page, page.locator('.detail-panel .close-button'), 'detail-close', testInfo, { vertical: true })
    await page.locator('.detail-panel .close-button').click()
    await expect(page.locator('.detail-panel')).toHaveCount(0)
  })
}