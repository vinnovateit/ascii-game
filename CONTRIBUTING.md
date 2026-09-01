# Contributing to ascii-game

This repository is a Rust ASCII game with a normal executable and a hot-reload development binary. Follow `BUILD.md` rather than the placeholder npm text that remains in the README template.

## Build and test

The checked-in toolchain file selects Rust 1.98 with rustfmt and Clippy:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run --bin main
```

For hot reload, run `cargo run --bin hot` and rebuild the library with `cargo build --lib` after changing game logic.

## Change guidelines

- Keep reusable state and game-loop behavior in the library; keep launch/reload behavior in `src/bin/`.
- Add unit tests for deterministic state transitions and boundary conditions.
- Keep terminal output legible at the supported window sizes.
- Do not commit `target/` or other generated build artifacts.

## Pull requests

Include formatting, Clippy, and test results. Describe the manual gameplay path used to verify rendering and input changes.
