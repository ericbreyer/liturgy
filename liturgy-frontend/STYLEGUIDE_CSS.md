# Current CSS Conventions

`src/main.ts` imports `src/styles/main.css` once. It loads `global.css` for
tokens/reset and `liturgical.css` for shared domain primitives. Do not import
global/domain CSS from component style blocks: that duplicates scoped rules.

`App.vue` owns page gutters, maximum width and the app shell. `PageLayout.vue`
is a shrinkable wrapper without additional horizontal gutters. Views own grids
and detail panels; `LiturgicalTable.vue` owns intentional internal scrolling.

Use `min-width: 0` on shrinkable flex/grid children and `minmax(0, 1fr)` on
equal grid tracks. Wrap long values. Keep Month's seven weekday columns at every
width and put full feast labels in the detail panel. Do not conceal layout bugs
with page-level overflow hiding. Bound overlays by viewport dimensions and scroll
their body while keeping close controls visible. Native date inputs should remain
labelled and use at least 16px text on mobile.

Validate with `npm run type-check`, `npm run build`, `npm run test:run` and
`npm run test:playwright`. Install browser engines with
`npx playwright install chromium webkit`. Browser tests auto-start the real local
backend and Vite server. They cover all eight routes at widths 320, 360, 390, 480,
640, 768, 769, 1024, 1280, 1440 and 1920, plus 568x320 and 1024x600 landscape
viewports in Chromium/WebKit. Screenshots/traces are evidence, not pixel-diff
baselines. Physical devices, browser zoom and every possible size remain separate.
