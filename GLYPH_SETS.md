# ASCII versus hybrid vocabulary experiment

ASCII remains the default and the accepted reference from `fa8d875`. No camera,
scene, geometry, lighting, depth, coverage, shadow threshold, dithering, microtexture,
edge decision or style default changed. Both modes first produce the same canonical
ASCII cells. A final resolver changes only their output characters.

## Review captures

Each pair is **ASCII on the left, hybrid on the right**, at 120×40 cells with the
unchanged `pixatool` preset. These are font previews using DejaVu Sans Mono, not
terminal screenshots. Preview colors illustrate two themes; the application sets
no foreground or background color.

| View | Dark preview | Light preview | Final text: ASCII / hybrid |
| --- | --- | --- | --- |
| Doorway | [Compare](visual/glyph-sets/comparisons/doorway-dark.png) | [Compare](visual/glyph-sets/comparisons/doorway-light.png) | [ASCII](visual/glyph-sets/ascii/doorway/frame.txt) / [hybrid](visual/glyph-sets/hybrid/doorway/frame.txt) |
| Crates | [Compare](visual/glyph-sets/comparisons/crates-dark.png) | [Compare](visual/glyph-sets/comparisons/crates-light.png) | [ASCII](visual/glyph-sets/ascii/crates/frame.txt) / [hybrid](visual/glyph-sets/hybrid/crates/frame.txt) |
| Stairs | [Compare](visual/glyph-sets/comparisons/stairs-dark.png) | [Compare](visual/glyph-sets/comparisons/stairs-light.png) | [ASCII](visual/glyph-sets/ascii/stairs/frame.txt) / [hybrid](visual/glyph-sets/hybrid/stairs/frame.txt) |

Visual inspection: doorway and crate boundaries become more continuous and a little
cleaner. Dark-face stippling, dense floor texture, silhouettes and depth separation
remain recognizable as the same renderer. Shade blocks are rare accents rather than
surface fills, although individual accents can look like bright specks. Stairs have
the largest change: continuous horizontal strokes clarify step boundaries but make
the image more ruled and diagrammatic. Replacing double-stroke `=` with single-stroke
`─` also reduces apparent weight. The reference's irregular text texture remains
closer in ASCII; hybrid is an optional readability tradeoff, not an established
visual improvement. Keep ASCII as the reference pending user review.

## Vocabulary and width verification

The eight additions are `░ ▒ ▓ █ ─ │ ╱ ╲`. Every one returns width **1** under
`unicode-width 0.2.2`; no combining characters, Braille or wide glyphs are permitted.
The library is the only added dependency. Rust and existing dependencies stay pinned.

| Canonical character | Hybrid result | Selection |
| --- | --- | --- |
| `\|` | `│` | Always |
| `/`, `\` | `╱`, `╲` | Always |
| `=` | `─` | Always |
| `:` | `░` | Deterministic spatial hash, 1/32 of eligible positions |
| `x`, `*` | `▒` | 1/32 |
| `#`, `%` | `▓` | 1/32 |
| `@` | `█` | 1/64 |
| Everything else, including spaces | Unchanged | Always |

The resolver has no new geometry or edge classification: it respects the canonical
mapper's decisions, including uses of `=` within surface texture. This limitation
is intentional for a vocabulary-only comparison. Both presets, custom ramps and
debug views share this resolver. There is no time-dependent randomness.

Interactive hybrid launch additionally prints each candidate and queries the
terminal cursor position to verify an actual advance of one column. A terminal
reporting a different advance, or failing to answer, causes a clean exit with a
suggestion to use `--glyph-set ascii`. ASCII needs no probe. Width checks cannot
guarantee font glyph availability, stroke appearance or freedom from font overhang.
Headless captures use library widths because there is no terminal to probe.

Presentation now compares persistent `char` cells and encodes changed row spans
as UTF-8. Cursor positions and row slicing count cells, never UTF-8 bytes. Empty
cells stay literal spaces; output remains theme-native.

## Measurements

Space positions are identical, not merely the total occupancy. No occupancy target
was used to tune the resolver. At 120×40:

| View | Occupancy, both modes | ASCII / hybrid density proxy | Unicode cells | Shade-block cells | ASCII / hybrid UTF-8 bytes |
| --- | --- | --- | --- | --- | --- |
| Doorway | 74.688% | 0.32043 / 0.30664 | 618 | 40 (0.83%) | 4840 / 6076 |
| Crates | 83.792% | 0.34581 / 0.33433 | 534 | 41 (0.85%) | 4840 / 5908 |
| Stairs | 85.104% | 0.37044 / 0.33683 | 1295 | 22 (0.46%) | 4840 / 7430 |

Density is a fixed glyph-weight diagnostic, not measured font ink or perceived
brightness. Hybrid shade weights are .25/.5/.75/1, line weights .32; ASCII weights
are unchanged. Telemetry retains canonical density so the upstream pipeline stays
comparable. `glyphs.txt` reports the resolved vocabulary's proxy separately.
Byte counts include newlines and exclude ANSI controls; full frames cost 22–54%
more bytes in hybrid. Changed-span traffic depends on camera movement.

Release CPU render + mapping + resolver, median of three alternating 3000-frame
doorway trials on this host (terminal output excluded):

| Size | ASCII | Hybrid | Difference |
| --- | --- | --- | --- |
| 120×40 | 0.718 ms | 0.760 ms | +0.042 ms / 5.8% |
| 200×60 | 1.773 ms | 1.875 ms | +0.102 ms / 5.8% |

[Raw benchmark results](visual/glyph-sets/performance.txt) include all trials and
checksums. Interactive PTY runs remained approximately 59–60 FPS in both modes.
PTY tests emulate cursor reports; real emulator/font appearance still needs local
review. The runtime probe performs that terminal's width check on every hybrid launch.

## Run and reproduce

```sh
cargo build --release --locked
target/release/main --glyph-set ascii
target/release/main --glyph-set hybrid

# Identical fixed viewpoints; replace hybrid with ascii for the reference.
target/release/main --glyph-set hybrid --camera-preset doorway
target/release/main --glyph-set hybrid --camera-preset crates
target/release/main --glyph-set hybrid --camera-preset stairs

# Final selected vocabulary on stdout, without ANSI escapes.
target/release/main --glyph-set hybrid --camera-preset doorway --snapshot --width 120 --height 40 > /tmp/doorway-hybrid.txt

# Save source buffers and the resolved output.
target/release/main --glyph-set hybrid --camera-preset doorway --capture-frame /tmp/doorway-hybrid --width 120 --height 40

# Regenerate all six captures and both-theme comparisons; requires ImageMagick
# and DejaVu Sans Mono for the previews, no extra Python packages.
uv run --no-project python scripts/glyph_compare.py
```

Capture `frame.txt` is the final selected vocabulary. `ascii.txt` is always the
canonical pure-ASCII intermediate, including in hybrid captures. `glyph-density.pgm`
and `glyphs.txt` describe the resolved output; the original luminance, depth, edges,
density, style and metrics files retain their previous meanings. All three paired
captures have byte-identical source/intermediate files.

## Validation and changed files

35 tests pass, including the 29 prior tests. Added checks cover the whitelist,
single-column widths, resolver identity, identical space masks, all three exact
paired snapshots, repeatability, conservative block use, UTF-8 row dimensions,
span encoding and unchanged default CLI output. The PTY smoke covers navigation,
tuning, debug modes, resize, clean exits, theme-native ANSI output, eight width
probes and restoration on wide-glyph or unanswered-probe rejection.

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
uv run --no-project python scripts/terminal_smoke.py
```

Changed implementation: `src/render/glyphs.rs` (new resolver),
`src/render/terminal.rs` (cell-based UTF-8 output and width probe),
`src/render/capture.rs` (resolved capture), `src/render/mod.rs`, `src/app.rs` (CLI
and plumbing), `Cargo.toml`, `Cargo.lock`. Validation/evidence changes:
`tests/glyph_sets.rs`, `scripts/terminal_smoke.py`, `scripts/visual_preview.py`,
`scripts/glyph_compare.py`, `.gitattributes`, `visual/glyph-sets/**`.
Documentation: this file and `BUILD.md`. Rendering/postprocessing source, presets,
camera poses, scene geometry, collision, input and hot reload are unchanged.

Next decision is visual acceptance of this optional vocabulary on the user's
terminal. Mesh import and better geometry remain a separate future milestone.
