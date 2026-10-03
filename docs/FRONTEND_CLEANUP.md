# Frontend Cleanup And Responsive Validation

Completed 2026-10-02. Existing theme and liturgical colors were preserved;
pre-existing Ordo and other user changes were not reverted.

## Changes

- Domain CSS loads once through `src/styles/main.css`, rather than being
  re-imported and scoped inside components. Removed duplicate feast, panel and
  visibility rules; documented stylesheet ownership.
- App shell owns page gutters and maximum width. Removed nested horizontal
  padding, duplicate `#app` markup and overly broad main/footer selectors.
- Navigation wraps without clipping; calendar dropdowns stay within their
  container; date controls use a shrinkable mobile grid and WebKit input rules.
- View grids and search results fit narrow containers. Month retains seven
  columns. Ordo values wrap/stack on phones. Wide comparison tables scroll
  internally rather than widening the document. Month details fit short screens.
- Search, Week and Month requests guard against stale responses and clean up
  cancellation. Week and Month date keys no longer shift through UTC conversion.
- Month SEO is registered during synchronous setup, fixing runtime head-plugin
  errors that prevented details from opening. Detail dates now match feast dates
  across positive/negative UTC offsets.
- Restored the frontend test baseline: isolated Vitest from Playwright, supplied
  head-plugin setup, filled empty suites and preserved one-day rolling Week
  navigation in its expectations.
- Added real-backend responsive browser tests and automatic local server startup.
  Vite now proxies API calls to the Rust backend on port 3000.

## Verified Results

| Check | Result |
| --- | --- |
| `npm run build` (includes type-check) | Passed |
| `npm run test:run` | 114 tests passed in 11 files |
| `npm run test:playwright` | 42 tests passed in Chromium and WebKit |
| Frontend source whitespace check | Passed |

Production CSS changed from 118.41 KB after the initial import cleanup to
64.52 KB at completion; gzip size changed from 12.81 KB to 10.35 KB.

## Browser Coverage

All eight routes are visited with real backend content at widths 320, 360, 390,
480, 640, 768, 769, 1024, 1280, 1440 and 1920 with a 900px height, plus 568x320
and 1024x600 landscape viewports. Calendar views use all five calendars; Ordo uses
its supported 1962 calendar.

Assertions cover horizontal document overflow, route/control bounds, nonblank
loaded content, uncaught script errors and unexpected API status failures.
Interaction coverage includes mobile navigation, calendar deselection/reselection,
date input/buttons, real search results and Month detail opening/closing. Month
date regression tests run in America/Los_Angeles and Asia/Tokyo.

Screenshots were reviewed for narrow mobile, desktop and short-landscape detail
layouts. Generated screenshots/traces live under `test-results/`; they are not
checked-in pixel-diff reference baselines.

## Run Locally

From `liturgy-frontend/`:

```sh
npx playwright install chromium webkit
npm run test:run
npm run test:playwright
```

The browser runner starts the backend and Vite server or reuses existing servers
outside CI. VS Code tasks also start them separately for interactive development.
The frontend is available at `http://127.0.0.1:5173/`.

## Limits

Representative breakpoint tests cannot establish correctness at every possible
size. Physical iOS/Android devices, native picker behavior, Firefox, browser zoom,
screen readers and arbitrary extreme content were not exhaustively tested.
Internal horizontal scrolling remains intentional for dense comparison tables;
Month's compact cell labels truncate, with full text available in details.