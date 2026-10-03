import { defineConfig, devices } from '@playwright/test'
import { fileURLToPath } from 'node:url'

const workspacePath = fileURLToPath(new URL('..', import.meta.url))

export default defineConfig({
  testDir: 'tests/playwright',
  timeout: 30_000,
  expect: { timeout: 5000 },
  fullyParallel: true,
  workers: 2,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: 'list',
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL || 'http://127.0.0.1:5173',
    actionTimeout: 0,
    trace: 'on-first-retry',
    viewport: { width: 1280, height: 800 },
    screenshot: 'only-on-failure',
  },
  webServer: [
    {
      command: `cargo run --manifest-path ../Cargo.toml -p liturgy-backend -- --calendar-data-dir "${workspacePath}/calendar_calc/calendar_data" --ordo-rules-dir "${workspacePath}/ordo/rules" --host 127.0.0.1 --port 3000`,
      url: 'http://127.0.0.1:3000/api/calendars',
      reuseExistingServer: !process.env.CI,
      timeout: 180_000,
    },
    {
      command: 'npm run dev -- --host 127.0.0.1 --port 5173 --strictPort',
      url: 'http://127.0.0.1:5173',
      reuseExistingServer: !process.env.CI,
      timeout: 60_000,
    },
  ],
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
})
