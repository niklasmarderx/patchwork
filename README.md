# patchwork

[![CI](https://github.com/niklasmarderx/patchwork/actions/workflows/ci.yml/badge.svg)](https://github.com/niklasmarderx/patchwork/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust stable](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

Understand and apply diffs — colored output, context-aware, built in Rust.

## Commands

```bash
# Explain what a diff does in plain English
patchwork explain changes.patch
git diff | patchwork explain -

# Colored side-by-side diff between two files
patchwork diff old.rs new.rs

# Apply a patch to files on disk
patchwork apply changes.patch --root ./my-project

# Preview without writing
patchwork apply changes.patch --dry-run
```

## Install

```bash
cargo install --git https://github.com/niklasmarderx/patchwork
```

## Example output

```
● src/main.rs
  +1 / -1 line(s) across 1 hunk(s)
  Hunk 1 (line 3): replaces 1 line(s) with 1
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
