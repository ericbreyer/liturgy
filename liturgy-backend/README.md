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
