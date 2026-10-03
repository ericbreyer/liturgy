# Repository Guide

Surveyed 2026-10-02 against the current working tree. This is an orientation,
not an exhaustive correctness or security audit. Existing Ordo, frontend,
snapshot, and cache-fingerprint changes were already present and left intact.

Subsequent frontend cleanup on the same date repaired the test baseline, CSS
ownership, responsive layouts, request races and Month date/SEO issues described
below. Current results: 114 unit tests and 42 Chromium/WebKit browser tests pass;
production build/type-check passes. See [FRONTEND_CLEANUP.md](FRONTEND_CLEANUP.md)
for current coverage and limitations. The survey findings below are historical.

## Big Picture

Liturgy is a data-driven liturgical calendar and office application. The core
is a five-crate Rust workspace; a Vue SPA presents multiple calendar traditions
side by side. Calendar definitions and office rules live in TOML, not a database.

```text
calendar_calc/calendar_data/*.toml
              |
              v
types <-- calendar_calc --> generated year calendars
  ^            |                    |
  |            v                    v
  +---------- ordo ----------> liturgy-backend (Axum)
               ^                    |
               |                    v
         ordo/rules/          liturgy-frontend (Vue)

cross-proc-cache: filesystem cache used by calendar tests
chant/: GABC and rendered HTML assets
divinum-officium/: separate upstream Perl submodule
```

The Rust backend does not invoke Divinum Officium's Perl engine in the inspected
request paths. Do not confuse the upstream submodule with the application's
calendar or Ordo implementation.

## Layout And Ownership

| Area | Responsibility | Start Here |
| --- | --- | --- |
| `types/` | Shared serialized domain types, liturgical units, ranks and flags | [types/src/lib.rs](../types/src/lib.rs) |
| `calendar_calc/` | Date rules, seasons, feast precedence, transfers, commemorations and year assembly | [calender.rs](../calendar_calc/src/calender.rs), [year_calendar_builder.rs](../calendar_calc/src/calender/year_calendar_builder.rs) |
| Calendar data | Definitions for 1954, 1962, monastic, Ordinary Form and US extensions | [54.toml](../calendar_calc/calendar_data/54.toml), [ef.toml](../calendar_calc/calendar_data/ef.toml), [of.toml](../calendar_calc/calendar_data/of.toml) |
| `ordo/` | Office-component selection, proper/common/ordinary fallbacks and Vespers | [ordo_repo.rs](../ordo/src/ordo_repo.rs), [rule_provider.rs](../ordo/src/rule_provider.rs), [toml_rule_provider.rs](../ordo/src/toml_rule_provider.rs) |
| `liturgy-backend/` | CLI configuration, HTTP handlers, in-memory caches and frontend static serving | [main.rs](../liturgy-backend/src/main.rs), [backend.rs](../liturgy-backend/src/web/backend.rs) |
| `liturgy-frontend/` | Calendar browsing, comparisons, search, novenas and Ordo display | [router/index.ts](../liturgy-frontend/src/router/index.ts), [services/api.ts](../liturgy-frontend/src/services/api.ts) |
| `cross-proc-cache/` | Fingerprinted file cache with cross-process locking | [src/lib.rs](../cross-proc-cache/src/lib.rs) |
| Release tooling | Crate publication and image deployment | [RELEASE.md](RELEASE.md), [publish-crates.yml](../.github/workflows/publish-crates.yml), [Dockerfile](../Dockerfile) |

The spelling `calender` is established in Rust module paths. Search using that
spelling when looking for implementation code.

## Design

### Calendar And Ordo

- Calendar computation separates generic calendar definitions from generated
  year calendars. Rank-specific resolution implements different rubrics rather
  than assuming one shared precedence table.
- The backend loads definitions, then lazily generates and caches year calendars
  by calendar name and year. Runtime caching is in memory; `cross-proc-cache` is
  a separate test-support concern.
- Ordo consumes calendar day descriptions and selects office components through
  a `RuleProvider` abstraction. `TomlRuleProvider` is the current filesystem
  implementation; this is an existing extension point, not a reason to add a
  database.
- Vespers support is explicitly rank-specific. The generic implementation in
  [ordo/src/lib.rs](../ordo/src/lib.rs#L85) returns an unsupported error outside
  `DayRank62`; UI and API changes should preserve that limitation explicitly.
- JSON envelopes use success/data/error fields. Frontend API types are maintained
  in TypeScript separately from Rust types, so contract changes need both sides
  checked.

### Frontend

- Views cover Today, Week, Month, Search, Nerd comparisons, Novena, Ordo and About.
- Shared composables manage calendar selection and date navigation. Route query
  parameters carry dates and calendar choices; no Pinia/Vuex dependency is used.
- The API client uses native `fetch`. Cancellation is supported for day requests,
  but is not applied consistently across all workflows.
- The visual vocabulary is domain-specific: liturgical colors, feast metadata,
  day cards, tables and responsive calendar layouts.
- [STYLEGUIDE_CSS.md](../liturgy-frontend/STYLEGUIDE_CSS.md) describes global tokens,
  domain styles and component-scoped CSS. Archived/stub styles also exist, so
  check actual imports before changing a similarly named stylesheet.
- Week and month loads fan out into per-date, per-calendar requests. This is
  simple, but increases request volume as selected calendars increase.

## Dependencies And Toolchain

These are manifest requirements, not a claim about the newest available releases.

| Layer | Main Dependencies |
| --- | --- |
| Rust domain | Chrono 0.4, Serde 1, TOML 0.9-family, bitflags, itertools |
| Rust API | Axum 0.8, Tokio 1.48 requirement, Tower 0.5, tower-http 0.6, Clap 4 |
| Ordo | Local `types` and `calendar_calc`, TOML, anyhow, nestify, Rayon |
| File cache | bincode 2.0.1, fs2 0.4.3, tempfile, Serde |
| Frontend runtime | Vue ^3.5.18, vue-router ^4.5.1, @vueuse/head ^0.8.2 |
| Frontend build | Vite ^7.0.6, TypeScript ~5.8.0, vue-tsc ^3.0.4 |
| Testing | Rust insta/test-case; frontend Vitest ^3.2.4 and Playwright ^1.55.0 |

- Rust editions are 2024 except the cache crate's 2021. The workspace uses
  resolver 3; use a toolchain supporting those choices.
- Frontend Node requirement: `^20.19.0 || >=22.12.0`.
- Both Cargo and npm lockfiles are tracked. Several Rust dependency requirements
  are broad (`*` or `>=`); lockfile discipline matters.
- Docker uses Node and Rust build stages, produces an x86_64 musl binary, and
  runs it as a non-root user in Alpine. The container copies calendar data,
  Ordo rules and built frontend assets, not a database.

## Tests And Current Health

Checks actually run during this survey:

| Check | Result |
| --- | --- |
| `cargo check --workspace --locked --quiet` | Passed; unused-code/import/variable warnings, notably in Ordo |
| `npm --prefix liturgy-frontend run type-check` | Passed |
| `npm --prefix liturgy-frontend run test:run` | Failed: 70 passed, 4 failed tests; 4 additional suite-loading failures |

The frontend failures are specific:

1. Two Week navigation tests expect seven-day movement, while the current
   [composable](../liturgy-frontend/src/composables/useDateNavigation.ts#L57)
   deliberately routes Week through one-day movement. Decide the intended UX
   before changing code or assertions.
2. Today snapshot and race tests fail during setup because the head plugin is
   not supplied. The race test therefore does not currently verify its behavior.
3. Vitest collects [today.spec.ts](../liturgy-frontend/tests/playwright/today.spec.ts),
   which belongs to Playwright, not Vitest.
4. [formatLocation.test.ts](../liturgy-frontend/src/tests/utils/formatLocation.test.ts),
   [OrdoView.test.ts](../liturgy-frontend/src/tests/views/OrdoView.test.ts) and
   [api.ordo.test.ts](../liturgy-frontend/src/tests/services/api.ordo.test.ts)
   contain no runnable test suites.

Calendar tests use snapshots and filesystem caches across multiple traditions
and years. Ordo has a year-integration snapshot suite; backend integration tests
cover static serving. Playwright has a desktop Today snapshot with Chromium and
WebKit projects, but no configured automatic server startup.

Full Rust tests, browser tests, production image builds and dependency security
scans were not run. Editor diagnostics included apparent Rust type errors that
the compiler check did not reproduce; prefer compiler evidence over that snapshot.

## Existing TODOs

- [Root README](../readme.md): Rogation days; week counting at the end of time
  after Pentecost; the Sunday in the Nativity octave; novenas for superseded feasts.
- [Calendar TODO](../calendar_calc/todo.md): St Andrew's commemoration/transfer
  behavior on Sunday and Monday in 2025.
- [1962 rank logic](../calendar_calc/src/calender/feast_rank/feast_rank_62.rs#L359):
  Ember-day detection TODO.
- [Backend generation](../liturgy-backend/src/web/backend.rs#L744): JSON output
  is a placeholder returning `{}`.
- [Backend statistics](../liturgy-backend/src/web/backend.rs#L780): actual
  statistics calculation remains a TODO. Treat this endpoint as unfinished.

## Bugs And Risks Worth Following Up

These are source-backed findings, not changes made in this survey.

1. **Deployment credential in source.** [dockerbuild.sh](../dockerbuild.sh)
   contains a hard-coded deployment hook credential. Treat it as exposed, rotate
   it, and move its replacement to a secret store or environment variable. Its
   value is intentionally not reproduced here. Rotation is needed even if the
   script is subsequently cleaned up.
2. **Search cancellation does not protect result ownership.**
   [useSearch.ts](../liturgy-frontend/src/composables/useSearch.ts#L46) replaces a
   shared controller, does not pass its signal to `searchFeasts`, and checks the
   current shared controller after awaiting. An older search can consequently
   overwrite a newer search. Clearing an in-flight search also does not reset
   `loading`. Reproduce with deferred API responses before fixing.
3. **Week/month stale-result protection is incomplete.**
   [WeekView.vue](../liturgy-frontend/src/views/WeekView.vue#L151) checks the
   mutable current controller rather than its request-local controller. Aborted
   older loads can still finish because per-request errors are caught, then
   publish stale/empty data or clear the newer loading state.
   [MonthView.vue](../liturgy-frontend/src/views/MonthView.vue#L204) guards loading
   cleanup by controller identity, but not the final writes into the shared
   month map. These are code-review findings, not browser reproductions.
4. **Date-only and timezone conventions differ.** WeekView constructs local
   midnight dates, adds fixed 24-hour intervals, then uses UTC `toISOString()`
   keys. Positive UTC offsets and DST boundaries deserve tests for shifted or
   repeated dates. Date navigation accepts syntactically valid dates without
   checking that the calendar date actually exists.
5. **Rule loading can succeed with missing data.**
   [toml_rule_provider.rs](../ordo/src/toml_rule_provider.rs#L76) discards several
   directory/file errors and logs then skips proper-rule parse failures. Top-level
   missing directories fail, but partial loads can return success. Relative paths
   also depend on a runtime `CARGO_MANIFEST_DIR` variable via `unwrap()`; standalone
   execution should use absolute paths or explicitly test that environment case.
6. **Calendar cache has no visible bound or single-flight guard.**
   [backend.rs](../liturgy-backend/src/web/backend.rs#L458) retains generated years
   in a map and computes misses between read and write locks. Concurrent misses
   can duplicate CPU work; requests for many years can grow memory usage. Assess
   accepted year ranges and realistic traffic before choosing a cache policy.
7. **Deployment reproducibility and script safety.**
   [Dockerfile](../Dockerfile) does not copy the Cargo lockfile or use `--locked`,
   uses floating builder tags, repeats `npm ci`, and masks an initial command's
   failure with `|| true`. Its `EXPOSE 3000` differs from the default `--port 8080`.
   The Alpine 3.18 runtime also warrants a lifecycle review. The release script
   has no fail-fast guard and proceeds through build, push and deployment commands.
   No image build or vulnerability scan was performed here.

## Working In The Repo

Run these from the repository root:

```sh
cargo check --workspace --locked
cargo test -p calendar_calc --lib
cargo test -p cross-proc-cache
npm --prefix liturgy-frontend run type-check
npm --prefix liturgy-frontend run test:run
```

For a built frontend served by Rust, use explicit paths instead of relying on
CLI defaults that depend on the current directory:

```sh
npm --prefix liturgy-frontend ci
npm --prefix liturgy-frontend run build
cargo run -p liturgy-backend -- \
  --calendar-data-dir "$PWD/calendar_calc/calendar_data" \
  --ordo-rules-dir "$PWD/ordo/rules" \
  --frontend-dir "$PWD/liturgy-frontend" \
  --host 127.0.0.1 --port 3000
```

The server appends `dist` to the frontend project path. Vite's API proxy block is
commented out, so a standalone Vite server needs its API routing checked before
assuming requests reach the Rust server. The backend README's minimal quick-start
does not fully describe these path/static-serving requirements.

Do not run [dockerbuild.sh](../dockerbuild.sh) as a harmless local build command:
it cleans Rust artifacts, pushes an image and triggers remote deployment.

## Suggested Order Of Work

1. Rotate the deployment credential and separate local image builds from publishing.
2. Restore a trustworthy frontend test baseline: isolate runners, fix plugin setup,
   resolve empty suites and agree on Week navigation semantics.
3. Add race/timezone regression tests, then fix request ownership and date keys.
4. Make missing Ordo rules observable and clarify unsupported calendar behavior.
5. Resolve the domain TODOs against authoritative rubric examples and snapshots.
6. Tighten image reproducibility, cache limits and incomplete API endpoints.