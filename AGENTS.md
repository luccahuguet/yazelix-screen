# Agent Guidelines

Shared Yazelix agent workflow and release policy live in the main repo:

- https://github.com/luccahuguet/yazelix/blob/main/AGENTS.md
- In sibling local checkouts, read `../yazelix/AGENTS.md` first

Only Yazelix Screen-specific guidance belongs here.

## Local Scope

- This repo owns the standalone `yzs` terminal animation engine and Rust crate.
- Main Yazelix owns integrated welcome/session policy and the `yzx screen` command surface.
- Keep renderer behavior usable from a plain terminal without a Yazelix session.

## Local Commands

- `cargo fmt --all -- --check`
- `cargo test`
- `cargo check --examples`
- `cargo run --bin yzs -- --help`
- `cargo run --example render_once`
- `nix build .#yzs --no-link`

## Integration Notes

Main Yazelix consumes the package and Rust crate through pinned child revisions. Publish child changes before updating main locks for coupled runtime work.
