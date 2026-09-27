# Visual convergence pass

This document records the first pass. Its hard-cutoff defaults and occupancy targets
are superseded by the [coverage and microtexture pass](PURE_ASCII.md). Historical
captures and measurements below are retained for comparison.

The default `pixatool` preset now favors empty space, stronger density bands and
near/far separation. This pass changes the ASCII backend and its controls only.
Projection, clipping, rasterization, lighting, depth storage, world geometry,
collision, movement and hot reload are unchanged. No dependencies were added or updated.

## Before/after evidence

Each comparison has **before on the left, after on the right**, with identical
120×40 buffers, camera pose, 70° vertical FOV and 0.5 cell width/height ratio.

| View | Dark preview | Light preview |
| --- | --- | --- |
| Doorway | [Compare](visual/comparisons/doorway-dark.png) | [Compare](visual/comparisons/doorway-light.png) |
| Crates | [Compare](visual/comparisons/crates-dark.png) | [Compare](visual/comparisons/crates-light.png) |
| Stairs | [Compare](visual/comparisons/stairs-dark.png) | [Compare](visual/comparisons/stairs-light.png) |

These are deterministic text/font previews, not screenshots of a terminal emulator.
Both sides are displayed using the same simulated theme for a fair glyph comparison;
the original runtime itself forced white on black. The new runtime emits only default
color/reset SGR, never foreground/background color selection or cell background fills.

| View | Old occupied cells | New occupied cells | Old non-space | New non-space | Old mean density | New mean density |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Doorway | 4196 | 2693 | 87.417% | 56.104% | 0.34037 | 0.26928 |
| Crates | 4578 | 2922 | 95.375% | 60.875% | 0.35590 | 0.31430 |
| Stairs | 4682 | 3281 | 97.542% | 68.354% | 0.39435 | 0.33507 |

Mean density includes empty cells and uses a fixed, font-independent ordinal ink
proxy in `src/render/capture.rs`: space=0; dot/apostrophe=0.12; comma/colon=0.22;
semicolon/exclamation/line glyphs=0.32; plus/equal=0.45; star/lowercase x=0.55;
uppercase X/dollar/ampersand=0.65; hash/percent=0.8; M/W/@/N=1. It is not a physical
font coverage measurement. Occupied cells are stronger on average despite lower
whole-frame density. Actual glyph weight depends on the terminal font.

Raw luminance and depth captures are byte-identical before/after. The upper dark
planes lose their punctuation carpet; the doorway becomes a clearer gap; crates
read as silhouettes against the floor; stair treads retain bright horizontal bands.
Far walls use lighter-density glyphs than foreground floors and stair treads.

The baseline is signed commit `bb96b78`, which adds capture instrumentation to the
old mapper without changing its behavior. `visual/before/` retains its source
buffers and text; `visual/after/` retains the new captures and exact style metadata.

## Processing order

1. Raw luminance times linear exposure.
2. Black/white point remap and clamp.
3. Gamma as `intensity^(1/gamma)`; larger gamma lifts midtones.
4. Contrast about 0.5, then clamp.
5. Linearize the existing NDC depth using the camera's near/far values. Modulate by
   `1 / (1 + (depth_falloff * view_depth)^1.3)`. Missing geometry remains empty.
6. Estimate a clamped-neighbor 3×3 Sobel gradient on the toned, depth-modulated image.
7. Apply the hard background cutoff. Below it, always emit literal space.
8. Quantize into posterization levels, using subtle centered 4×4 Bayer perturbation
   at the quantizer input. Dither never runs on cells rejected by the cutoff.
9. Select the density family and a deterministic, limited variant. Flat areas get
   30% of the configured variation probability; modest gradients permit more;
   gradients above the edge threshold suppress variants. Bands below 0.4 remain canonical.
10. Optionally substitute `|`, `=`, `/`, or `\` only on already-visible cells where
    `edge_magnitude * edge_weight > edge_threshold`. No wireframe geometry is added.

Cutoff precedes dither so noise cannot repopulate negative space. The gradient is
computed before posterization so band boundaries do not become artificial outlines.
Only intensity and edge scratch arrays are added; both are reused across frames.

Six default density families: `space`, `.'`, `:;`, `xX`, `#%`, `MW@`.
A custom `--ramp` still supplies one character per family. Optional directional
edge glyphs are independent of that ramp. Screen-coordinate hashing has no time seed.

## Defaults and CLI parameters

The same small `AsciiStyle` struct drives CLI overrides, runtime tuning and captures.
`pixatool` is the intended default; `neutral` disables depth, texture and edge styling
for inspection. Explicit parameter flags override presets regardless of CLI order.

| Flag / parameter | pixatool | neutral | Range |
| --- | ---: | ---: | --- |
| `--exposure` | 1.05 | 1 | 0.1–4 |
| `--gamma` | 0.95 | 1 | 0.1–4 |
| `--contrast` | 1.6 | 1 | 0.1–4 |
| `--black-point` | 0.16 | 0 | 0–0.98 |
| `--white-point` | 0.82 | 1 | 0.02–1 |
| `--background-cutoff` | 0.20 | 0.08 | 0–1 |
| `--posterization-levels` | 6 | 8 | 4–12 integers |
| `--dither-strength` | 0.12 | 0 | 0–2 |
| `--texture-variance` | 0.35 | 0 | 0–1 |
| `--depth-falloff` | 0.065 | 0 | 0–0.5 per world unit |
| `--edge-weight` | 0.15 | 0 | 0–1 |
| `--edge-threshold` | 0.1 | 0.1 | 0.01–1 |

White point must exceed black point by at least 0.02. `--levels` and `--dither`
remain aliases. Six families are mapped across the configured quantization levels;
raising the level count changes thresholds, not the size of the character palette.

## Run and tune

```sh
cargo build --release --locked
cargo run --release --locked --bin main -- --preset pixatool
cargo run --release --locked --bin main -- --preset neutral --telemetry
```

Navigation remains WASD and mouse/arrows. Q/Esc/Ctrl+C exit. 1–5 retain the original
debug modes. T shows/hides telemetry. V toggles visual tuning and enables telemetry.
While tuning:

| Key | Action |
| --- | --- |
| `,` / `.` | Previous / next parameter (all 12) |
| `-` / `=` | Decrease / increase by one fine step |
| `_` / `+` | Decrease / increase by five steps |
| `r` | Reset all values to the selected preset; keep any custom glyph ramp |
| `p` | Switch neutral/pixatool and reset values |

Fine steps: exposure/gamma/contrast/dither/variance/edge weight=0.05;
black/white/cutoff/edge threshold=0.01; depth falloff=0.005; levels=1.
Changes apply on the next rendered frame. No navigation bindings are consumed.
Telemetry adds occupied cell count/percentage, density proxy, raw mean luminance,
mean view depth, preset, and selected tuning parameter/value. The two telemetry rows
are excluded from visual metrics and captures. Values are not shown in normal mode.
Runtime edits are session-local; use CLI flags to reproduce them on the next run.

## Fixed cameras and captures

| Preset | Position | Yaw | Pitch |
| --- | --- | ---: | ---: |
| doorway | (0, 1.65, 5) | 0 | 0 |
| crates | (-0.4, 1.65, 1.8) | -0.62 | -0.14 |
| stairs | (0, 1.65, -8) | 0 | -0.12 |

Angles are radians. `room` remains an alias for doorway; `landing` and `--view` remain supported.

```sh
cargo run --release --bin main -- --camera-preset doorway
cargo run --release --bin main -- --camera-preset crates
cargo run --release --bin main -- --camera-preset stairs

# Plain text only; diagnostics stay off stdout.
cargo run --release --bin main -- --snapshot --camera-preset doorway > doorway.txt
cargo run --release --bin main -- --snapshot --camera-preset crates > crates.txt
cargo run --release --bin main -- --snapshot --camera-preset stairs > stairs.txt

# All channels at fixed 120x40; no terminal needed.
target/release/main --camera-preset doorway --capture-frame /tmp/ascii-doorway
target/release/main --camera-preset crates --capture-frame /tmp/ascii-crates
target/release/main --camera-preset stairs --capture-frame /tmp/ascii-stairs
```

Each capture directory contains `ascii.txt`, `luminance.pgm`, `depth.pgm`,
`edges.pgm`, `density.pgm`, `metrics.txt`, and `style.txt`. PGM files are simple
8-bit grayscale images with one sample per cell. Depth displays
`1-sqrt(view_depth/far)`; metrics use unscaled world-unit depth. Edge visualization
is the gradient magnitude, independent of whether edge glyphs are enabled.
Capture mode always renders raw source lighting and final styled ASCII, even when
`--mode` requests a debug view. Captures and rotating benchmarks are separate runs.
`--width`, `--height`, `--fov` and `--cell-aspect` make the inputs explicit.

Optional font previews (Python standard library; ImageMagick for comparison PNGs):

```sh
uv run --no-project python scripts/visual_preview.py /tmp/ascii-doorway
uv run --no-project python scripts/visual_compare.py
```

To reproduce the old mapper in an isolated checkout:

```sh
git worktree add --detach /tmp/ascii-game-before bb96b78
cd /tmp/ascii-game-before
cargo run --release --locked --bin main -- --camera-preset doorway --capture-frame /tmp/before-doorway
cargo run --release --locked --bin main -- --camera-preset crates --capture-frame /tmp/before-crates
cargo run --release --locked --bin main -- --camera-preset stairs --capture-frame /tmp/before-stairs
```

## Validation and performance

27 Rust tests pass. Existing renderer correctness tests remain unchanged. The two
original exact snapshots were intentionally updated. New tests cover all three
camera views at 120×40, 200×60 and 1×1; exact default text at the capture size;
40–76% occupancy and 0.15–0.45 mean density at regular sizes; over one-third empty
cells in the top quarter; at least 20 percentage points less occupancy than baseline;
static-frame repeatability; depth fall-off; zero-falloff equivalence; cutoff/dither
invariance; edge gating; empty geometry; neutral mapping; and parameter validation.
Current raw luminance and depth are checked against the byte-identical baseline captures.

The PTY smoke test exercises tuning selection/adjustment/reset/preset switching,
all original movement/debug/resize/exit paths, terminal restoration and absence of
explicit color SGR. All four required Cargo checks pass, including Clippy with
warnings denied. The underlying 3D and hot-reload code is frozen.

| Optimized CPU render + ASCII, 1,000 frames | Before | After |
| --- | ---: | ---: |
| 120×40 | 0.344 ms/frame | 0.378 ms/frame |
| 200×60 | 0.836 ms/frame | 0.951 ms/frame |
| Interactive drained PTY | 59–60 FPS | 59–60 FPS |

These are indicative local runs, not statistical benchmarks. CPU timings include
the existing checksum and exclude terminal output/capture writes. Reproduce with:

```sh
target/release/main --benchmark 1000
target/release/main --benchmark 1000 --width 200 --height 60
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
uv run --no-project python scripts/terminal_smoke.py
```

## Files and remaining visual limits

[Exact changed-file manifest](visual/CHANGED_FILES.txt). Main implementation files:
`src/render/ascii.rs`, `style.rs`, `capture.rs`, `terminal.rs`, `mod.rs`,
`src/app.rs`, `src/viewpoints.rs`, and `src/lib.rs`. Tests, captures, preview scripts,
PGM binary attributes and documentation make up the remaining changes.

Hard posterization deliberately leaves visible bands; it is not smooth grayscale.
Very dark crate faces can merge into empty regions and read chiefly by silhouette.
Flat shading provides limited local texture, and very thin features remain limited
by terminal resolution. Screen-anchored variants are stable at rest but slide over
surfaces during motion. Gradient direction is cell-based, so diagonal glyphs are an
approximation. Preview weights depend on the font, and physical terminal performance
and theme contrast still deserve a manual check. This is a stylistic approximation
of the supplied reference, not a reconstruction of Pixatool's algorithm.

Next: accept or refine these same viewpoints on the target terminal/font. Only after
that visual acceptance should mesh import and better world geometry become the next
major milestone. No networking or gameplay work is part of this pass.
