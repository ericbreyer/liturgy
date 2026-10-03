# liturgy-frontend

Vue 3 frontend for the Liturgy calendar and Ordo application.

## Cycle Comparison

NerdView defaults to source sanctoral definitions rather than observed daily
winners. The comparison is sanctoral-only; This date
shows feasts assigned to that date in any selected calendar and their dates in
the other calendars. Full cycle compares the year's entire source catalog and
can sort by first date or name. Its search field filters names, descriptions,
titles, and ISO dates locally, ignoring case and accents. Search is hidden and
does not filter This date mode; returning to Full cycle retains the search.

The view preserves suppressed definitions and labels cross-calendar differences
as "Different date", not transfers. For example,
`/nerd?date=2026-10-03&calendars=ef,monastic,of-us` compares St. Therese on October 3
in EF/monastic with October 1 in OF-US. The selected catalog and scope are
shareable through the `scope=year` query parameter. Legacy `cycle=temporal`
links show the sanctoral comparison; temporal source data remains in the API.

Matching uses the normalized name and the entire normalized, order-independent
title set for every entry, plus the audited registry in `src/utils/feastIdentity.ts`.
Different titles are different identities unless an explicit name/title
equivalence is recorded. The registry links verified historical variants to
qualified names (such as Cyril of Alexandria/Jerusalem and Augustine of
Canterbury/Hippo), and handles confirmed aliases including Rosary, Carmel,
Lourdes, Sacred Heart, and the Servite founders. Original source titles remain
visible in each calendar's cell.
Separate celebrations (including Agnes's second feast and the Peter/Paul
commemorations) remain separate. Different saint rosters are not merged.
Identities are stable regardless of calendar selection; ranks do not identify
saints. Unlisted translations and alternate names are not inferred.
Full-cycle labels compare each feast's dates across calendars: Same date,
Different date, or Only in this calendar. They do not compare with the date picker.

## Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur).

## Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) to make the TypeScript language service aware of `.vue` types.

## Customize configuration

See [Vite Configuration Reference](https://vite.dev/config/).

## Project Setup

```sh
npm ci
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

The development server proxies `/api` to the Rust backend on port 3000. Start
the backend with explicit calendar-data and Ordo-rule paths, or use the repository's
`Liturgy: local backend` and `Liturgy: frontend dev` VS Code tasks.

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

## Tests

```sh
npm run test:run
npx playwright install chromium webkit
npm run test:playwright
```

Vitest only collects `src/tests/**/*.test.ts`. Playwright starts the local Rust
backend and Vite server automatically and reuses running servers outside CI.
Ports 3000 and 5173 must be available or already running this application.

The real-data browser suite covers all eight routes across phone, tablet,
desktop and short-landscape viewports in Chromium and WebKit. It checks overflow,
control bounds, runtime errors, search, navigation, calendar selection and Month
detail panels. Ordo is tested with the supported 1962 calendar.

See [STYLEGUIDE_CSS.md](STYLEGUIDE_CSS.md) for stylesheet ownership and responsive
layout conventions. Browser screenshots/traces are generated evidence rather
than pixel-diff baselines; physical-device and zoom testing remain separate.
