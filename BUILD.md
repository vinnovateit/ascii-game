# Build and run

This milestone is a CPU triangle renderer for an interactive monochrome terminal environment.
Use a Linux terminal with a monospace font, preferably at least 120 columns by 40 rows.
Rustup selects the checked-in Rust 1.98.1 toolchain. Cargo.lock pins dependencies.

```sh
cargo build --release --locked
cargo run --release --locked --bin main
```

Walk straight ahead through the framed doorway and continue down the hallway to climb
six steps onto the landing. Crates occupy the room on both sides. Turn around on the
landing to see the stairs and doorway from above.

## Controls

| Input | Action |
| --- | --- |
| W / S | Forward / backward |
| A / D | Strafe |
| Arrow keys | Yaw / pitch |
| Mouse motion or drag | Look (terminal reporting required) |
| 1 | Final ASCII |
| 2 | Unprocessed luminance mapped to a simple glyph ramp |
| 3 | Linearized depth mapped to a simple glyph ramp |
| 4 | Depth-tested triangle wireframe |
| 5 | Face lighting without material albedo |
| T | FPS, frame time, triangle counts, dimensions, position |
| Q / Esc / Ctrl+C | Exit and restore terminal |

Terminals with the keyboard enhancement protocol provide held-key/release tracking.
Legacy terminals provide press/repeat events only: movement expires 180ms after the
last event, so the initial keyboard repeat delay can cause a brief pause. Mouse
reporting is cell-based, bounded by the terminal window, and cannot provide unlimited
relative mouse capture. Arrow keys always provide a turning fallback. Focus loss
clears held inputs when reported by the terminal.

## Rendering options

See [PURE_ASCII.md](PURE_ASCII.md) for current visuals, coverage-aware mapping and review frames.
See [GLYPH_SETS.md](GLYPH_SETS.md) for the optional `--glyph-set hybrid` experiment,
paired captures and terminal width verification. `--glyph-set ascii` remains the default.
[VISUAL.md](VISUAL.md) retains the earlier tuning guide and historical comparisons.
The default is the textured, coverage-aware, theme-native `pixatool` preset. V enables tuning;
`,`/`.` select, `-`/`=` adjust finely, `_`/`+` adjust coarsely, R resets and P switches presets.


```sh
cargo run --release --bin main -- --telemetry --cell-aspect 0.5 --fov 70
cargo run --release --bin main -- --preset pixatool --shadow-threshold 0.18 --depth-falloff 0.025
cargo run --release --bin main -- --ramp ' .,:;+=xX$#%@'
cargo run --release --bin main -- --snapshot --view stairs --width 120 --height 40
cargo run --release --bin main -- --benchmark 1000 --width 200 --height 60
```

`--cell-aspect` is character width divided by character height; set it for your font.
FOV is vertical, in degrees. The default glyph families contain similar-density
variants selected sparingly by a deterministic screen-coordinate hash and local gradients. A custom ramp uses one
character per band. All output is monochrome. `--view room|stairs|landing` selects
an initial camera for interactive runs, snapshots, or benchmarks. `--mode 1..5`
selects an initial debug view. `--width` and `--height` control headless output;
interactive dimensions follow the terminal, capped at 500×240 cells. `--help` lists options.

## Pipeline and source layout

- `src/engine/`: right-handed camera, input, 120Hz fixed update, player-box collision.
- `src/render/mesh.rs`: indexed triangles and model transforms.
- `src/render/scene.rs`: world-space face normals, back-face culling, ambient plus directional diffuse light, perspective projection.
- `src/render/clipping.rs`: all six homogeneous frustum planes, before perspective divide.
- `src/render/rasterizer.rs`: pixel-center barycentric coverage and affine NDC Z-buffer depth.
- `src/render/framebuffer.rs`: persistent floating-point luminance/depth and physical-cell aspect.
- `src/render/ascii.rs`: exposure/levels → gamma → contrast → bounded depth modulation → coverage-aware shadow stippling → textured posterization → thinned structural edges.
- `src/render/terminal.rs`: raw mode, alternate screen, cursor/mouse/keyboard modes, persistent frame comparison and batched changed-row spans.
- `src/world/test_scene.rs`: room, hallway, doorway, crates, six stairs and raised landing; matching box colliders.
- `src/app.rs`: configuration, event processing, simulation/render scheduling, telemetry and headless validation.

Meshes can contain arbitrary triangles; collision currently uses separate axis-aligned
boxes. Flat lighting is deliberately simple. There are no textures, shadows, gameplay
systems or graphics engines. The renderer is statically owned; hot reload is kept as
an independent counter experiment, not a way to exchange renderer-owned Rust values.

## Experimental hot reload (Linux)

```sh
cargo build --lib
cargo run --bin hot
# In another terminal, after changing the counter library:
cargo build --lib
```

The hot executable accepts an optional shared-library path. ABI v1 uses an explicit
`extern "C"` version function and a `repr(C)` one-field state payload. Each candidate
is copied to a distinct file, validated, and swapped only after successful loading.
An incompatible or partially built candidate leaves the previous library active.
Symbols never outlive their owning library. Version the ABI whenever the payload or
function contract changes. Only load trusted local libraries.

## Validation

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
cargo build --release --locked
uv run --no-project python scripts/terminal_smoke.py
uv run --no-project python scripts/hot_smoke.py
```

The Python smoke scripts use only the standard library. The hot test also uses the
OS C compiler to exercise the actual C ABI. Snapshot fixtures are in `tests/snapshots/`.
See [RENDERER.md](RENDERER.md) for measured results and remaining limitations.
