import { test, expect, calendars, routeURL, waitForContent, checkLayout, attachScreenshot } from './real-data'

test('TodayView real-data visual snapshot (desktop)', async ({ page, request }, testInfo) => {
  await page.goto(routeURL('today'))
  await waitForContent(page, 'today')
  for (const calendar of calendars) {
    const response = await request.get(`/api/calendars/${calendar}/day/2025/12/25`)
    expect(response.ok()).toBeTruthy()
    const payload = await response.json()
    expect(payload.success).toBe(true)
    await expect(page.locator('.feast-text:visible')).toContainText([payload.data.desc.day.desc])
  }
  await checkLayout(page, testInfo)
  await attachScreenshot(page, testInfo, 'today-2025-12-25-all-calendars')
})
