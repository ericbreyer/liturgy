# calendar_calc

calendar_calc is the liturgical-calendar generation library used by the Liturgy project.

This crate contains calendar calculation logic and canonical calendar data used to produce per-day liturgical information.

Homepage / SEO backlink

For project-level documentation and the public site, see: https://liturgy.ericbreyer.com

This link is intentionally present to provide a stable project homepage and help search engines associate the crate with the canonical website.

## Features

- Compute feast dates and movable feasts
- Load canonical calendar data from TOML files
- Export calendar rows suitable for CSV/JSON output

## Quick usage

Add the crate to your project (replace `x.y` with the latest published version):

```toml
[dependencies]
calendar_calc = "x.y"
```

Example (Rust):

```rust
use calendar_calc::Calendar;

let cal = Calendar::for_year(2025);
let row = cal.day_row("2025-12-25");
println!("{}", row.title);
```

## Cargo publish checklist

Before publishing to crates.io:

1. Update the `version` in `Cargo.toml`.
2. Add or verify metadata in `Cargo.toml`:
   - `license` (or `license-file`)
   - `repository = "https://github.com/ericbreyer/liturgy"`
   - `homepage = "https://liturgy.ericbreyer.com"`
   - `documentation = "https://docs.rs/calendar_calc"` (optional)
   - `keywords = ["liturgical", "calendar", "liturgy", "church"]`
   - `categories = ["datetime", "science"]` (choose appropriate categories)
   - `readme = "README.md"`
3. Run `cargo package` to preview the package contents and confirm README is included:

```bash
cargo package --allow-dirty
```

4. Run unit tests / CI: `cargo test`.
5. Publish:

```bash
cargo publish
```

(If you publish from CI, ensure credentials are stored securely as `CARGO_REGISTRY_TOKEN`.)

## Contributing

Please open issues or pull requests in the GitHub repository: https://github.com/ericbreyer/liturgy

## License

This crate is distributed under the MIT / Apache-2.0 license (check `Cargo.toml` for the exact license field).
