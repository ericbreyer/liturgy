# types

Shared types used across the Liturgy workspace. This crate centralizes common data structures and DTOs shared by other crates in the workspace.

Project homepage:

https://liturgy.ericbreyer.com

## Usage

```toml
[dependencies]
types = "x.y"
```

## Publishing checklist

1. Bump `version` in `Cargo.toml`.
2. Ensure `readme = "README.md"` is in `Cargo.toml`.
3. Add `repository`, `homepage`, and `license` metadata.
4. Run packaging check:

```bash
cargo package --allow-dirty
```

5. Run tests and format checks:

```bash
cargo test
cargo fmt -- --check
```

6. Publish:

```bash
cargo publish
```

## License

See `Cargo.toml` for license details.
