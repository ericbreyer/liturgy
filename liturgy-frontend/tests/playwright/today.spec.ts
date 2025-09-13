import { test, expect } from '@playwright/test'

// Make sure the dev server is running at http://localhost:5173 before running these tests
test('TodayView visual snapshot (desktop)', async ({ page }) => {
  await page.goto('http://localhost:5173/')
  // Wait for the main app layout and a known element to be visible
  await page.locator('#app').waitFor({ state: 'visible' })
  // Optionally wait for a specific view header or content
  await page.waitForTimeout(500) // let fonts and styles settle
  // Take a full page screenshot for baseline comparison
  const screenshot = await page.screenshot({ fullPage: true })
  expect(screenshot).toMatchSnapshot('todayview-desktop.png')
})
