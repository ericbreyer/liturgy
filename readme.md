# Liturgy

This repository contains the Liturgy project: a liturgical calendar generator (Rust), a backend API, and a Vue 3 frontend.

Homepage: https://liturgy.ericbreyer.com

## Overview

- `calendar_calc/` — calendar computation library and canonical TOML calendar data.
- `ordo/` — helpers for liturgical titles, flags, and textual fallbacks.
- `types/` — shared types used across the workspace.
- `liturgy-backend/` — HTTP API server serving calendar data and the frontend `dist/`.
- `liturgy-frontend/` — Vite + Vue 3 SPA for browsing the calendar.

## Local Builds

Build the frontend and backend, then serve both locally:

```sh
./scripts/local-build.sh
```

The script installs frontend dependencies with `npm ci`, runs the production
frontend build/type-check, builds Rust with the Cargo lockfile, and starts the
backend at `http://127.0.0.1:3000/` with explicit calendar, Ordo and frontend paths.
Run it from any directory; press Ctrl+C to stop. It does not clean, publish,
push images or deploy.

```sh
./scripts/local-build.sh --port 3001    # Use another port
./scripts/local-build.sh --release      # Optimized Rust build
./scripts/local-build.sh --build-only   # Build without starting a server
./scripts/local-build.sh --skip-install # Reuse existing node_modules
./scripts/local-build.sh --help
```

Requires Cargo with Rust 2024 edition support and Node.js `^20.19.0` or
`>=22.12.0`. Dependencies are installed by default; use `--skip-install` only
when existing dependencies match the npm lockfile. The default bind address is
local-only; use `--host 0.0.0.0` explicitly to expose it on the network.

## Publishing crates

Some workspace crates are intended to be published to `crates.io`. Individual crates include `calendar_calc`, `ordo`, `types`, `cross-proc-cache`, and `liturgy-backend`. See each crate's README for `cargo publish` instructions and metadata expectations.

## TODO

- rogation days
- strange behavior counting weeks at the end of time after pentecost
- strange behavior concerning sunday in nativity octave
- novena view won't show superseded feasts
