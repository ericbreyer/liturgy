# liturgy-backend

The liturgy-backend crate provides an HTTP API and supporting logic for the Liturgy project. It serves precomputed calendar data and static assets (including the frontend distribution) and exposes endpoints consumed by the SPA.

Project homepage / SEO backlink

https://liturgy.ericbreyer.com

This README intentionally includes the site link to provide a canonical project homepage for crates.io and search engines.

## Quick start

```bash
cargo run -p liturgy-backend --release
```

By default the server serves files in `./dist/` from the frontend build and exposes JSON calendar endpoints on `/api`.

## Source cycles

`GET /api/calendars/{name}/cycles/{cycle}/{year}` returns source feast definitions
in the existing `{success, data, error}` envelope. Calendars are `54`, `ef`,
`monastic`, `of`, and `of-us`; cycles are `sanctoral` and `temporal`; civil years
must be between 1 and 9999.

Each entry includes `name`, `description`, `date_rule`, nullable ISO `date`,
`rank`, `color`, and `titles`. Dates resolve the source rule for the requested
civil year without applying precedence or transfers. Suppressed feasts remain
present; an unresolvable date is null. This endpoint does not build or cache an
observed year calendar.

Fixed-date rules default to sanctoral and movable rules to temporal. Root
`[cycle_overrides]` tables in the calendar TOML files classify exceptions by exact
source name, including fixed Christmas feasts and movable saints' feasts.
Extension overrides merge with the base calendar. The temporal catalog contains
source feast rules, not the generated weekday, Sunday, or season schedule.

## Publishing checklist

- Update `version` in `Cargo.toml`.
- Ensure `readme = "README.md"` present in `Cargo.toml`.
- Add metadata fields (`repository`, `homepage`, `documentation`, `keywords`, `categories`).
- Run tests and check formatting:

```bash
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
```

- Preview package:

```bash
cargo package --allow-dirty
```

- Publish:

```bash
cargo publish
```

## License

See `Cargo.toml` for license information.
