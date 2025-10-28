# cross-proc-cache

`cross-proc-cache` is a small utility crate used inside the Liturgy workspace to provide a simple cross-process caching abstraction.

Project homepage / backlink:

https://liturgy.ericbreyer.com

Keeping the homepage link helps tie crates.io entries back to the public site for discoverability.

## Usage

Add to your Cargo.toml:

```toml
[dependencies]
cross-proc-cache = "x.y"
```

Example:

```rust
use cross_proc_cache::Cache;

let cache = Cache::new("liturgy-cache");
cache.insert("key", "value");
```

## Packaging & publish checklist

- Bump `version` in `Cargo.toml`.
- Ensure `README.md` is referenced in `Cargo.toml` (`readme = "README.md"`).
- Add metadata: `repository`, `homepage`, `license`, `keywords`, and `categories`.
- Verify package contents with `cargo package`:

```bash
cargo package --allow-dirty
```

- Run tests and linters:

```bash
cargo test
cargo fmt -- --check
```

- Publish:

```bash
cargo publish
```

## License

Check `Cargo.toml` for the declared license.
