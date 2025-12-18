# Releasing with cargo-release

This project uses `cargo-release` to manage version bumps, tags, and publishing of workspace crates.

Prerequisites

- Install `cargo-release`:

```bash
cargo install cargo-release
```

- Ensure you have a crates.io API token available if publishing from CI or local machine.
  Export it as `CARGO_REGISTRY_TOKEN` or run `cargo login <token>` locally.

- Make sure `git` user/email are configured and you have push rights for the repo.

Quick local dry-run

```bash
# Validate what would happen without publishing
./scripts/release-workspace.sh --dry-run
```

Full interactive release

```bash
# This will prompt for confirmation and then perform releases according to release.toml
./scripts/release-workspace.sh
```

Notes

- `release.toml` in the repo root configures workspace mode and an opinionated tag format `v{version}`.
- `cargo-release` will update each crate's `version` field, create commit(s), tag(s), and push them. It will then publish crates to crates.io if `publish = true` (in `release.toml`).
- For CI, set `CARGO_REGISTRY_TOKEN` as a repository secret and run `cargo release --workspace --no-confirm` in a protected workflow.

CI suggestions

- Use the workflow to run `cargo release --workspace --dry-run` on pull requests to validate.
- For actual publishing on tags, run `cargo release --workspace --no-confirm` with `CARGO_REGISTRY_TOKEN` available.

Caveats

- The workspace must publish crates in dependency order. `cargo-release` in workspace mode inspects dependency relationships and will publish in the correct order.
- If you previously added `version` to local path dependencies for packaging convenience, you may remove them before publishing; `cargo-release` will manage versions appropriately. However, retaining explicit versions is fine as long as they are kept consistent.
