# Development on NixOS

From the repository root:

```sh
nix develop
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo build --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo run --locked --bin main
```

The `x86_64-linux` flake pins nixpkgs and provides NixOS-patched rustup, a native
compiler, pkg-config, Python 3.13 and uv. `rust-toolchain.toml` selects Rust
1.98.1 with rustfmt and Clippy. Cargo keeps the existing application manifest
and lockfile; the flake does not update application dependencies.

Toolchains and downloads use `~/.cache/vinnovateit/rustup` and
`~/.cache/vinnovateit/cargo`. Once dependencies are cached, append `--offline`
to Cargo commands when working without a network.

For the existing hot-reload experiment, build the library with
`cargo build --locked --lib`, then run `cargo run --locked --bin hot`.
See BUILD.md for usage. This tooling-only change does not modify game logic,
repair existing source issues or include unpublished feature-branch work.
