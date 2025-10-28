# Liturgy

This repository contains the Liturgy project: a liturgical calendar generator (Rust), a backend API, and a Vue 3 frontend.

Homepage: https://liturgy.ericbreyer.com

## Overview

- `calendar_calc/` — calendar computation library and canonical TOML calendar data.
- `ordo/` — helpers for liturgical titles, flags, and textual fallbacks.
- `types/` — shared types used across the workspace.
- `liturgy-backend/` — HTTP API server serving calendar data and the frontend `dist/`.
- `liturgy-frontend/` — Vite + Vue 3 SPA for browsing the calendar.

## Publishing crates

Some workspace crates are intended to be published to `crates.io`. Individual crates include `calendar_calc`, `ordo`, `types`, `cross-proc-cache`, and `liturgy-backend`. See each crate's README for `cargo publish` instructions and metadata expectations.

## TODO

- rogation days
- strange behavior counting weeks at the end of time after pentecost
- strange behavior concerning sunday in nativity octave
- novena view won't show superseded feasts
