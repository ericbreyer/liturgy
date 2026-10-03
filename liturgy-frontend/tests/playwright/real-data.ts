import { test as base, expect, type Page, type TestInfo, type Locator } from '@playwright/test'

export const calendars = ['54', 'ef', 'monastic', 'of', 'of-us']
export const stableDate = '2025-12-25'
export const routes = ['today', 'week', 'month', 'search', 'nerd', 'novena', 'ordo', 'about'] as const
export type Route = (typeof routes)[number]

export const test = base.extend<{ runtimeGuard: void }>({
  runtimeGuard: [async ({ page }, use, testInfo) => {
    const errors: string[] = []
    const pending: Promise<void>[] = []
    page.on('pageerror', error => errors.push(`pageerror ${page.url()}: ${error.message}`))
    page.on('response', response => {
      if (!new URL(response.url()).pathname.startsWith('/api/') || response.status() < 400) return
      pending.push((async () => {
        const body = await response.text().catch(() => '')
        const unsupported = /\/api\/ordo\/vespers\/(?!ef\/)[^/]+\//.test(response.url())
          && /unsupported|not supported/i.test(body)
        if (!unsupported) errors.push(`API ${response.status()} ${response.url()}: ${body}`)
      })())
    })
    await page.route(/https?:\/\/[^/]*(google-analytics|googletagmanager|plausible|umami|cloudflareinsights|analytics)[^/]*\//,
      route => route.abort())
    await use()
    await Promise.all(pending)
    await testInfo.attach('runtime-errors', { body: JSON.stringify(errors, null, 2), contentType: 'application/json' })
    expect.soft(errors, 'Uncaught script errors and unexpected API failures').toEqual([])
  }, { auto: true }],
})
export { expect }

export function routeURL(route: Route) {
  return `/${route}?date=${stableDate}&calendars=${route === 'ordo' ? 'ef' : calendars.join(',')}`
}

export async function waitForContent(page: Page, route: Route) {
  const root = page.locator('main.app-main > *').first()
  await expect(root).toBeVisible()
  const ready: Record<Route, string> = {
    today: '.liturgy-content:visible', week: '.liturgy-content:visible',
    month: '.calendar-day.has-feast', search: '.search-input',
    nerd: '.feast-row, .no-feasts', novena: '.novena-category',
    ordo: '.ordo-item', about: '.hero-title',
  }
  await expect(page.locator(ready[route]).first()).toBeVisible()
  await expect(page.locator('.background-loading, .loading-calendars, main .loading')).toHaveCount(0)
  await expect(page.locator('main .error:visible, main .error-display:visible')).toHaveCount(0)
  await expect(root).not.toHaveText('')
  if (route === 'today' || route === 'week') {
    const days = route === 'today' ? 1 : await page.locator('.liturgical-day-card:visible, .liturgical-row:visible').count()
    expect(days, 'Rendered day rows/cards').toBeGreaterThan(0)
    await expect(page.locator('.liturgy-content:visible')).toHaveCount(days * 5)
    await expect(page.locator('.no-data:visible, .no-data-cell:visible')).toHaveCount(0)
  }
  if (route === 'ordo') await expect(page.locator('.ordo-item')).toHaveCount(7)
  await page.evaluate(() => document.fonts.ready)
}

export async function checkBounds(page: Page, locator: Locator, selector: string, testInfo: TestInfo,
  options: { vertical?: boolean; unclipped?: boolean } = {}) {
  const measurements = await locator.evaluateAll((elements, options) => elements
    .filter(element => element.getClientRects().length && getComputedStyle(element).visibility !== 'hidden')
    .map(element => {
      const rect = element.getBoundingClientRect()
      const parent = element.parentElement?.getBoundingClientRect()
      return {
        selector: element.tagName.toLowerCase() + '.' + [...element.classList].join('.'),
        text: element.textContent?.trim().slice(0, 100), left: rect.left, right: rect.right,
        top: rect.top, bottom: rect.bottom, width: rect.width,
        viewportWidth: innerWidth, viewportHeight: innerHeight,
        scrollWidth: element.scrollWidth, clientWidth: element.clientWidth,
        fault: rect.left < -1 || rect.right > innerWidth + 1
          || (options.vertical && (rect.top < -1 || rect.bottom > innerHeight + 1))
          || (options.unclipped && (element.scrollWidth > element.clientWidth + 1
            || (parent && (rect.left < parent.left - 1 || rect.right > parent.right + 1)))),
      }
    }), options)
  const faults = measurements.filter(measurement => measurement.fault)
  if (faults.length) {
    const name = `${new URL(page.url()).pathname.slice(1)}-${page.viewportSize()?.width}-${selector.replace(/[^a-z0-9]/gi, '_')}`
    await testInfo.attach(`${name}-measurements`, { body: JSON.stringify(faults, null, 2), contentType: 'application/json' })
    await attachScreenshot(page, testInfo, `${name}-screenshot`)
  }
  expect.soft(faults, `${page.url()} ${selector} bounds: ${JSON.stringify(faults)}`).toEqual([])
}

export async function checkLayout(page: Page, testInfo: TestInfo) {
  const dimensions = await page.evaluate(() => ({
    width: innerWidth, documentWidth: document.documentElement.scrollWidth,
    bodyWidth: document.body.scrollWidth,
  }))
  if (dimensions.documentWidth > dimensions.width + 1 || dimensions.bodyWidth > dimensions.width + 1) {
    await testInfo.attach('document-overflow', { body: JSON.stringify({ url: page.url(), ...dimensions }), contentType: 'application/json' })
    await attachScreenshot(page, testInfo, `${new URL(page.url()).pathname.slice(1)}-overflow`)
  }
  expect.soft(dimensions.documentWidth, `${page.url()} document width ${JSON.stringify(dimensions)}`).toBeLessThanOrEqual(dimensions.width + 1)
  expect.soft(dimensions.bodyWidth, `${page.url()} body width ${JSON.stringify(dimensions)}`).toBeLessThanOrEqual(dimensions.width + 1)
  await checkBounds(page, page.locator('main.app-main, main.app-main > *'), 'route-root', testInfo)
  await checkBounds(page, page.locator('.date-controls, .date-controls button, .date-picker'), 'date-controls', testInfo)
  for (const control of await page.locator('.date-controls button:visible').all()) {
    await expect.soft(control).toHaveAccessibleName(/.+/)
  }
  const picker = page.locator('.date-picker:visible')
  if (await picker.count()) await expect.soft(picker).toHaveAccessibleName(/Select (date|month)/)
}

export async function attachScreenshot(page: Page, testInfo: TestInfo, name: string) {
  const path = testInfo.outputPath(`${name}.png`)
  await page.screenshot({ path, fullPage: true, animations: 'disabled' })
  await testInfo.attach(name, { path, contentType: 'image/png' })
}