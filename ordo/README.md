# ordo

The `ordo` crate provides common logic related to liturgical titles, rubrics, and textual fallbacks used throughout the Liturgy project.

This crate is authored as part of the Liturgy project; the canonical website and project home is:

https://liturgy.ericbreyer.com

Please keep that link in the README so crates.io pages link back to the main project website for discoverability.

## Highlights

- Utilities for constructing canonical titles and flags for liturgical days
- Helpers used by calendar generation and rendering pipelines

## Add to your project

```toml
[dependencies]
ordo = "x.y"
```

## Publishing notes (cargo publish)

1. Ensure `Cargo.toml` version is bumped.
2. Verify `Cargo.toml` metadata is present and correct:
   - `license`
   - `repository = "https://github.com/ericbreyer/liturgy"`
   - `homepage = "https://liturgy.ericbreyer.com"`
   - `documentation` (optional)
   - `readme = "README.md"`
3. Run checks:

```bash
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
cargo package --allow-dirty
```

4. Publish:

```bash
cargo publish
```

If you expect to publish multiple crates from this workspace, prefer a CI-based release to ensure reproducibility.

## License

See `Cargo.toml` for license information.
