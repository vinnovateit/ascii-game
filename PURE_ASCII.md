# Coverage and microtexture review

This is the second visual pass, following `f1b8552`. The first pass erased too much
covered geometry and left surviving planes as nearly uniform character fields.
The new default restores surface presence and fine texture without changing the
3D pipeline, geometry, collision, terminal colors, dependencies, or toolchain.

## Three deterministic frames for review

All frames use the same camera poses, 120×40 cells, 70° vertical FOV and 0.5 cell
aspect as the preceding pass. These are monospace previews of actual ASCII text,
not terminal-emulator screenshots. The colors in the previews simulate themes;
the application continues to use terminal defaults.

| View | New frame | Plain ASCII | Previous/new comparison |
| --- | --- | --- | --- |
| Doorway | [Preview](visual/pure-ascii/after/doorway/dark.png) | [Text](visual/pure-ascii/after/doorway/ascii.txt) | [Dark](visual/pure-ascii/comparisons/doorway-dark.png) / [Light](visual/pure-ascii/comparisons/doorway-light.png) |
| Crates | [Preview](visual/pure-ascii/after/crates/dark.png) | [Text](visual/pure-ascii/after/crates/ascii.txt) | [Dark](visual/pure-ascii/comparisons/crates-dark.png) / [Light](visual/pure-ascii/comparisons/crates-light.png) |
| Stairs | [Preview](visual/pure-ascii/after/stairs/dark.png) | [Text](visual/pure-ascii/after/stairs/ascii.txt) | [Dark](visual/pure-ascii/comparisons/stairs-dark.png) / [Light](visual/pure-ascii/comparisons/stairs-light.png) |

Comparison images have the preceding renderer on the left and this pass on the right.
Previous captures are retained under `visual/pure-ascii/before/`; the older initial
renderer captures remain in `visual/before/`. Current raw luminance and depth remain
byte-identical to the original fixed views.

## Visual assessment against the supplied Pixatool reference

The reference combines stable large architectural forms with dense, irregular small
marks on their surfaces. This pass restores that relationship: a wall remains a wall
while its characters vary. The shaded crate face is no longer an empty cutout; the
corridor frame remains distinct within the wall; and stair treads retain their strong
horizontal rhythm while risers carry mixed texture. Long single-glyph slabs are broken
up without adding time-random flicker.

The first experimental edge pass produced doubled outlines. Non-maximum suppression
now thins gradient ridges before directional glyph substitution. This keeps stronger
structural boundaries without making the whole image wireframe.

There are still differences from the reference: 120×40 has much coarser character
sampling, the scene has simple flat lighting, and our cell-based diagonal marks can
look schematic. The enclosed test room has geometry across the whole viewport,
including its dark ceiling; those regions now receive sparse surface marks instead
of being treated as empty sky. Their increased occupancy is intentional. No occupancy
percentage was used as the tuning target or acceptance criterion.

## Mapping changes

1. Coverage comes from finite NDC depth in `[0,1]`. Uncovered samples always become
   literal spaces in the final ASCII backend, regardless of exposure, dither or edges.
2. Exposure, black/white remap, gamma and contrast still produce the surface tone.
3. Depth modulation is gentler and bounded:
   `0.4 + 0.6 / (1 + (depth_falloff * view_depth)^1.3)`.
4. `shadow_threshold` selects a sparse-mark regime, not geometry rejection. Below
   it, deterministic spatial stippling retains marks even for fully black covered
   geometry. Increasing the threshold makes more surfaces use this shadow treatment.
5. Tone receives a small bounded deterministic perturbation. Ten quantization levels
   and spatial mixing of neighboring density families soften broad band boundaries.
6. Family variants use the existing spatial hash. A local 3×3 luminance standard
   deviation modulates their strength; even flat lit planes have a texture baseline.
7. Sobel gradients of toned luminance and logarithmic view depth identify structural
   boundaries, including depth discontinuities with little lighting contrast. The
   stronger gradient is thinned with a consistent plateau tie rule. Strong ridges can
   use `|`, `=`, `/` or `\`, including on the dark side of covered geometry.
8. Variant strength is reduced on these ridges so the surface texture does not obscure
   their direction. Uncovered pixels can never receive edge marks.

All glyphs are printable ASCII. The families are space, punctuation (`.,'` plus
backtick), `:;-`, `=+~`, `x*X`, `#%&`, and `MW@`. Scratch arrays are reused each frame;
there are no per-cell allocations or temporal random values.

## Current pixatool defaults

```text
exposure=1.05          gamma=0.95          contrast=1.4
black_point=0.09       white_point=0.90    shadow_threshold=0.18
posterization_levels=10                  dither_strength=0.30
texture_variance=0.90  depth_falloff=0.025
edge_weight=0.50      edge_threshold=0.12
```

`--shadow-threshold` replaces the misleading name `--background-cutoff`. The old
flag remains an alias but now has the new semantics. V and the existing tuning
controls still work; the parameter label now says `shadow-threshold`.
`neutral` remains available for inspection. A zero texture-variance setting removes
family/adjacent-band variation, but covered dark surfaces still use spatial stippling.

## Reproduce and capture

```sh
cargo build --release --locked
target/release/main --camera-preset doorway
target/release/main --camera-preset crates
target/release/main --camera-preset stairs

target/release/main --snapshot --camera-preset doorway > doorway.txt
target/release/main --snapshot --camera-preset crates > crates.txt
target/release/main --snapshot --camera-preset stairs > stairs.txt

target/release/main --camera-preset doorway --capture-frame /tmp/ascii-doorway
target/release/main --camera-preset crates --capture-frame /tmp/ascii-crates
target/release/main --camera-preset stairs --capture-frame /tmp/ascii-stairs
uv run --no-project python scripts/visual_compare.py visual/pure-ascii
```

The capture bundle still includes luminance, depth, edge and density images, final
ASCII, metrics and all style settings. Edge images show the unthinned combined
magnitude; only the selected ridges affect final glyphs. Existing `visual/after/`
artifacts and the original comparison script default remain historical evidence.

## Validation and limits

29 tests pass. Tests now distinguish covered black geometry from uncovered black
background, require dark surface presence, verify texture diversity and static-frame
stability, preserve exact snapshots, and compare source buffers against the original
captures. Previous occupancy targets and tests that required erasing dark geometry
were removed. Coverage decisions are tested with finite, infinite, NaN and out-of-range
depth, including maximum dither/edge settings. Existing 3D correctness tests are unchanged.
Formatting, Clippy with warnings denied, `cargo check`, and the PTY smoke tests pass.

Optimized render plus ASCII conversion: 0.678 ms/frame at 120×40 and 1.697 ms/frame
at 200×60, over 1,000 rotating-camera frames. The 120×40 baseline in this session was
0.375 ms/frame. The extra depth-gradient and local-variation work costs CPU time but
remains comfortably below the frame budget. Interactive PTY output remains 59–60 FPS.
Measurements exclude capture writes and actual terminal-emulator display latency.

The surface pattern is screen-anchored and stable at rest, so it slides over geometry
when moving. Very low-resolution details remain approximate, and edge directions can
look stylized. These frames are presented for visual review, not as a claim that the
reference aesthetic is now exactly reproduced. The next step is evaluating this
pure-ASCII balance in the target terminal before any Unicode experiment.
