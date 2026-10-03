# ordo

The `ordo` crate provides common logic related to liturgical titles, rubrics, and textual fallbacks used throughout the Liturgy project.

This crate is authored as part of the Liturgy project; the canonical website and project home is:

https://liturgy.ericbreyer.com

Please keep that link in the README so crates.io pages link back to the main project website for discoverability.

## Highlights

- Utilities for constructing canonical titles and flags for liturgical days
- Helpers used by calendar generation and rendering pipelines

## Roman 1962 Component Resolution

The current office resolver targets Roman 1962 Vespers. Calendar occurrence and
concurrence decisions remain separate from selecting each office component's source.
This does not establish rules for the monastic office or other editions.

`RuleProvider::get_proper_rule` returns the first matching key in the supplied
priority order, with its ID, optional common, and full TOML bytes in
`MatchedProperRule`. Custom providers must supply the same rule format as the
filesystem provider. The repository parses that data and selects `[first_vespers]`
or `[vespers]` according to the requested office.

For each component, an explicit proper source overrides the office template.
An omitted component or section retains the template's fallback; a missing
`[first_vespers]` section does not borrow `[vespers]`. Provider common metadata
takes precedence over the rule's common, followed by the existing title-based
common fallback. Invalid matched rule data fails with rule-specific context
rather than silently discarding the proper.

The `OfficeComponentFamily` abstraction keeps the merger independent of the hour's
component layout, while `RuleProvider` keeps retrieval independent of filesystem
storage. Seasonal coverage, commemoration handling, and concurrence still require
further work; a passing component-resolution test is not full rubrical validation.

Run the focused baseline without year-snapshot updates:

```sh
cargo test -p ordo --lib --locked
```

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
