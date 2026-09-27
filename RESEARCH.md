# Glyph reconstruction research

**Result:** source detail matters more than replacing scalar mapping with a patch
objective alone. The strongest practical matched candidate combines normalized
shape and gradient correlation, world-attached source variation, revised lighting,
exact density-bound search and modest hysteresis. It is an interesting opt-in mode,
but is **not clearly better than the accepted fast mapper for gameplay**. Fast keeps
cleaner architectural boundaries, better motion stability and much lower cost.
The default remains fast; no accepted render/postprocess code or presets changed.

Final comparisons (left to right: accepted fast / matched / enriched-source control):

| View | Dark theme | Light theme | Matched text |
| --- | --- | --- | --- |
| Doorway | [Compare](visual/research/final/doorway-dark.png) | [Compare](visual/research/final/doorway-light.png) | [Frame](visual/research/final/matched/doorway/frame.txt) |
| Crates | [Compare](visual/research/final/crates-dark.png) | [Compare](visual/research/final/crates-light.png) | [Frame](visual/research/final/matched/crates/frame.txt) |
| Stairs | [Compare](visual/research/final/stairs-dark.png) | [Compare](visual/research/final/stairs-light.png) | [Frame](visual/research/final/matched/stairs/frame.txt) |

The PNGs are font previews, not terminal screenshots. The application uses default
terminal colors. [Temporal playback](visual/research/temporal/review.html) is a
self-contained, synchronized 180-frame review for four camera paths. Open it in a
modern browser; it needs no server or network.


## A — Baseline and hypotheses

Baseline commit: `6e5f5ed`. All 35 existing tests and fmt/strict Clippy/check pass.
Fast ASCII and hybrid stay unchanged and the default stays fast. Source captures
for doorway, crates and stairs are in `visual/research/baseline/`.

The supplied reference is preserved in `visual/research/reference.png`. It is a
492×269 compressed video frame, not an original lossless Pixatool output. Excluding
the bottom 19 rows (timestamp), its mean gray is 0.187; 35.0% of pixels are below
16/255 and almost none exceed 192/255. These are image-pixel statistics, NOT terminal
cell occupancy or evidence of the original foreground settings.

Mean absolute pixel differences at horizontal lags 1..12 have minima at 4, 8, 12;
vertical lags have minima at 6 and 12. Visual inspection also suggests a roughly
4×6 pixel text grid. That would be approximately 123×42 cells over this crop: the
apparent fine sampling may not require vastly more cells than our 120×40 captures.
Compression, rescaling and the source scene prevent confident glyph identification.
No inference here establishes Pixatool's actual algorithm. The author's
[November 2017 release notes](https://kronbits.itch.io/1bit-game-assets/devlog/14912/pixatool-116)
confirm the addition of ASCII art processing. The official
[product page](https://kronbits.itch.io/pixatool) describes ASCII/ANSI image export,
not text export. These support the image-postprocess framing, but do not establish
patch matching as the historical implementation.

Reference forms have dense, repetitive microstructure and strong silhouettes.
Our raw luminance image has strong shapes but large perfectly flat planes, no
surface texture and only one luminance per triangle. A shape matcher cannot invent
source structure. Independent hypotheses:

1. Supersampled cell patches improve local boundary orientation over scalar mapping.
2. Pure pixel MSE overvalues background/coverage and collapses flat regions.
3. Normalized shape/gradient metrics help edges but cannot fix flat interiors alone.
4. World-attached source detail supplies useful patch information without arbitrary
   screen-space glyph noise.
5. Fixed foreground intensity constrains image reconstruction more than glyph count.
6. Font mismatch and temporal popping may outweigh still-image improvements.

The experiment will separate matcher, source detail, sampling and intensity changes.
Use identical camera poses and physical viewport aspect, measured glyph masks and
source/reconstruction captures. Exact reconstruction metrics are atlas-specific,
not a direct similarity score against this geometrically different reference.

Baseline source preview: `visual/research/baseline/source.png` (enlarged luminance).
No rendering changes at this checkpoint. No dependency or toolchain changes planned.

## C/D — Initial matcher failures

Implemented isolated research runner with MSE, normalized correlation plus density,
gradient correlation, compact features, foreground scale and scale-plus-offset.
All use the same supersampled image (6×10 samples/cell) and measured atlas coverage.
Source tone is `(raw-.08)/.72`, clamped, scaled to the atlas maximum mean coverage.
This puts the image into the density range a default-foreground glyph can express.

`visual/research/comparisons/initial.png` shows fast / normalized / gradient. Results:

- Fixed-intensity binary MSE chose spaces everywhere. With target pixels below .5,
  adding a white glyph pixel increases squared error. This is a real objective
  mismatch, not evidence that an empty image is visually best.
- Normalized/gradient/feature methods preserve scene forms but produce repeated
  punctuation/brackets/ampersands across flat planes. They are worse than fast.
  Gradient correlation only helps at the relatively few boundary patches.
- Optimal foreground scale alone overwhelmingly chooses the densest `@` shape,
  modulating its intensity. Structure is not recovered from flat input.
- Adding an offset nearly reproduces the flat grayscale image (doorway MSE .000154),
  but requires background fills. Its low error is misleading for a theme-native
  terminal renderer; this option is analysis-only and rejected for presentation.
- Full gradient matching at 120×40 costs ~38 ms including ~11 ms source rasterization;
  it is already too costly for 60 FPS. Reduced sampling and shortlisting need testing.

Captures, actual-font previews, per-glyph counts and atlas reconstruction errors are
stored for all seven variants and all three cameras. Pixel error penalizes the useful
high-frequency ink of real glyphs; it cannot rank art direction by itself. The initial
source supplies no interior structure for shape matching. Next isolate source detail.

## E/G — Source, sampling and search experiments

[Experiment index](visual/research/experiments.csv) contains 107 captured method/view
combinations. Each research capture includes the frame, source, reconstruction,
distribution and metrics. Later captures also store their exact command. The initial
flat-source runs predate source changes and use the original runner defaults.

| Hypothesis / variation | Observation | Decision |
| --- | --- | --- |
| Normalized shape versus gradient correlation | Similar on flat input; gradients follow some actual patch features once source structure exists | Keep combined metric for matched |
| Compact feature matching | Cheaper than full search but often chooses rows of `=` and brackets; loses fine orientation | Keep as research control |
| 14 versus 34 glyphs | Small set produces stronger repeated `x/#` fields and loses shape vocabulary | Keep 34 for matched |
| World-space noise alone | Entropy rises, but flat original lighting still populates dark planes uniformly | Insufficient alone |
| Lower ambient plus directional/fill lighting | Creates useful plane contrast and negative space; also risks suppressing dark geometry | Retain only in experimental source |
| Noise plus lighting | Strongest practical improvement in source structure; still looks procedural | Matched finalist |
| Initial depth-neighborhood contact darkening | Incorrectly shades perspective slopes as though occluded | Discard this formula |
| Plane-relative contact darkening | More plausible contact accents, but small visual gain for its cost | Research only, off in finalist |
| Larger-neighborhood contrast | Adds some separation and detail but also extra texture/streaks; additional pass costs time | Research only |
| Image gradient-magnitude boost | Increases dense accents at boundaries but also fills surfaces with bright texture | Reject as final default |
| Source-driven near-equivalent Top-K | Slightly richer stills, severe popping under motion | Off in finalist |
| Left-neighbor coherence penalty | Reduces entropy and encourages horizontal runs | Off in finalist |
| 2×4 / 3×6 / 4×8 / 6×10 / 8×16 sampling | More samples help boundaries and template shape, with steep raster/source cost | 3×6 compromise; 2×4 performance option |
| Calibrated DejaVu versus original bitmap atlas | Changes density ordering and glyph choice; confirms meaningful font dependence | Optional offline calibration |
| 8×16 calibrated matching | Some richer local shape, no decisive macro improvement, ~105–125 ms at 120×40 | Too expensive for practical use |
| MSE at higher exposure | Removes the all-space failure but becomes large uniform dense fields (doorway entropy 1.46 bits) | Reject for final mode |
| Foreground scale / foreground plus background offset | Lower reconstruction error does not imply the desired aesthetic | Diagnostics only |
| Existing fast mapper on enriched source | Faster than matching, but revised lighting makes walls/crate faces too faint | Retain as opt-in control, not default |

Relevant visual evidence:
[initial matcher failures](visual/research/comparisons/initial.png),
[source variations](visual/research/comparisons/source.png),
[sampling](visual/research/comparisons/sampling.png),
[feature versus correlation](visual/research/comparisons/metrics.png),
[font calibration](visual/research/comparisons/calibration.png),
[late candidates](visual/research/comparisons/late-candidates.png),
[exterior silhouette](visual/research/comparisons/silhouette.png).

The exterior pose views the **existing** scene from outside. No mesh was added or
changed. Genuine uncovered background stays empty in both methods. This also
exposes a comparison confound: the reference shows an exterior object with void,
whereas our three principal views are inside a room filled by walls and ceiling.
Matching its empty-image percentage would erase legitimate geometry.

Historical source-v1 directories are `world_noise`, `contact`, `lighting`,
`combined`, `context`, `font_combined`, `compact`, and `topk`. Their captures are
retained failures, not current golden fixtures. V1 used unfiltered
`.55*noise(uv*3)+.45*noise(uv*23)` and contact darkening from positive view-depth
differences. V2 filters unresolved noise frequencies and measures occluder separation
relative to the reconstructed surface plane. Rerunning v1 command files with the
current source implementation is therefore not an exact historical reproduction.
The rejected formula is not retained as an interactive backend. Early records that
lack `source_ms` include source processing in `convert_ms`; use the final staged
performance table for comparable timings.

The 4/8-candidate coarse shortlist changed some full-search choices. The final
search instead uses a mathematically valid lower bound: the nonnegative density
term alone can reject candidates whose total score cannot win. Start with the
nearest-density glyph, evaluate survivors, then use stable atlas-order ties.
Tests compare this bounded search against an unpruned search, including Top-K.
The optimization retains the full objective's answer rather than accepting a
quality loss from a fixed shortlist.

## F — Motion and temporal stability

Three seconds, 180 samples at a **simulated 60 Hz**, for each path:

- Yaw: 0.12 rad/s.
- Forward: 0.6 world units/s.
- Strafe: 0.6 units/s close to the doorway, from x=-0.6, z=-5.8.
- Stairs: 1.2 units/s starting at z=-9.6; the existing collision/step-up code advances
  at 120 Hz, so this actually ascends the six steps rather than interpolating a ramp.

Mean changed glyphs divided by the union of non-space cells in adjacent frames:

| Mode | Yaw | Forward | Strafe | Stairs |
| --- | ---: | ---: | ---: | ---: |
| Accepted fast | 1.61% | 0.42% | 1.84% | 2.96% |
| Matched, no history | 14.35% | 7.00% | 22.89% | 31.60% |
| Matched, hysteresis .035 | 4.89% | 1.92% | 10.02% | 12.83% |
| Matched, Top-K=3 | 34.06% | 22.65% | 42.93% | 49.70% |

Some change is legitimate movement of world-attached detail. Fast's screen-attached
texture naturally changes less, so this metric is not a pure flicker score. However,
Top-K adds substantial avoidable instability. It remains a research option only.
Hysteresis retains the old glyph when its score is within .035 of the new winner,
provided patch mean and depth have not changed substantially. Empty coverage clears
immediately; resizing and debug-mode changes reset history.

Consecutive-frame inspection shows more stable surface marks with hysteresis and
continued movement of geometry boundaries. No persistent ghost silhouette appears
in the inspected frames. This is a cell-history heuristic, **not motion-compensated
reprojection**: fine patterns can stick and then change, especially on stairs and
near doorframes. It is not as calm as fast. See the
[yaw filmstrip](visual/research/temporal/yaw-filmstrip.png),
[stairs filmstrip](visual/research/temporal/stairs-filmstrip.png), and full
[playback](visual/research/temporal/review.html). Stationary-camera tests have zero
change. Fixed-pose captures start with fresh history and are reproducible.

## H/I — Practical candidate and performance

The practical matched backend uses an original 34-glyph atlas at 3×6 samples per
cell. It combines density, normalized patch correlation and normalized gradient
correlation, with reliability reduced for nearly flat patches. Noise comes from
world coordinates reconstructed using the existing depth buffer, not screen hashes.
The source pass uses two frequencies with footprint-based attenuation. There is no
Top-K variation, neighbor penalty, contact pass or larger-neighborhood sharpening in
the finalist. Hysteresis is .035. Source black/white points are .055/.7, noise
amplitude .22, revised lighting enabled. No time-random noise is used.

Release measurements on this host: medians of three serial 120-frame slow-yaw runs.
Each stage median is computed independently, so rounded stage medians need not add
to the median total. Capture/reconstruction/metrics and terminal presentation are
outside these CPU timings. `source` includes enrichment, tone conversion and any
cell averaging; `convert` is the ASCII mapper. The glyph resolver/presentation cost
is included in the separate PTY observations, not the staged research runner.

| Size | Mode | Raster ms | Source ms | Convert ms | CPU ms | Observed PTY FPS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 120×40 | fast | .264 | 0 | .559 | .832 | 60 |
| 120×40 | matched 3×6 | 3.617 | 7.475 | 5.597 | 16.674 | 57.5 |
| 120×40 | matched 2×4 | 1.651 | 3.095 | 3.975 | 8.721 | 60 |
| 120×40 | enriched control | 3.552 | 7.453 | .572 | 11.578 | 60 |
| 200×60 | fast | .556 | 0 | 1.309 | 1.865 | 60 |
| 200×60 | matched 3×6 | 9.016 | 19.890 | 13.725 | 42.740 | 23 |
| 200×60 | matched 2×4 | 4.033 | 8.315 | 9.690 | 22.028 | 42 |
| 200×60 | enriched control | 8.793 | 19.899 | 1.378 | 30.070 | 32 |
| 240×80 | fast | .885 | 0 | 2.221 | 3.106 | not measured |
| 240×80 | matched 3×6 | 14.221 | 33.233 | 21.513 | 68.932 | not measured |
| 240×80 | matched 2×4 | 6.392 | 13.930 | 15.228 | 35.368 | not measured |
| 240×80 | enriched control | 13.990 | 33.482 | 2.231 | 49.699 | not measured |

[All CPU trials](visual/research/performance/trials.csv) and
[PTY measurements](visual/research/performance/pty.json) are retained. The PTY test
reconstructs ANSI changed spans before reading telemetry; raw regex extraction can
mistake an update to `57fps` for `7fps` when the first digit was unchanged. Reported
FPS is the median of recent complete telemetry values during actual movement.
The tests also exercise all debug modes, 1×1 and ordinary resizing, input, exit,
terminal restoration and absence of explicit foreground/background colors.

3×6 is usable at 120×40 but does **not** reach the 30 FPS floor at 200×60 on this
host. Use 2×4 there. Even 2×4 becomes marginal at 240×80. The selected configuration
is a quality/cost compromise, not a universal performance claim. No threading or
GPU path was added. Source enrichment is now a larger cost than glyph matching;
optimizing the matcher alone will not solve the larger-terminal problem.

## J — Statistics and final visual review

Measured ink density comes from the **same 3×6 atlas** for all three finalists.
Entropy includes spaces. The MSE column compares all outputs with the same enriched,
PGM-quantized source and atlas, not with the geometrically different reference image.
Fast did not receive that enriched input, so this is a reconstruction diagnostic,
not a fair standalone score of the converter algorithm. The enriched control did
receive the same source as matched.

| View | Backend | Occupancy | Entropy bits | Atlas ink | Common-target MSE |
| --- | --- | ---: | ---: | ---: | ---: |
| Doorway | fast | 74.69% | 3.492 | .1077 | .02901 |
| Doorway | matched | 94.42% | 4.332 | .1395 | .03239 |
| Doorway | enriched | 62.67% | 3.250 | .0761 | .02539 |
| Crates | fast | 83.79% | 3.692 | .1154 | .03157 |
| Crates | matched | 98.12% | 4.351 | .1516 | .03682 |
| Crates | enriched | 63.38% | 3.222 | .0856 | .02764 |
| Stairs | fast | 85.10% | 3.719 | .1206 | .03301 |
| Stairs | matched | 99.38% | 4.594 | .1478 | .03481 |
| Stairs | enriched | 69.54% | 3.417 | .0737 | .02583 |

[Statistics CSV](visual/research/final/statistics.csv) and each finalist's
`statistics.json` contain full glyph distributions and MAE. Research captures also
include gradient reconstruction error. The original fast ordinal density telemetry
remains unchanged; it is not the calibrated atlas ink measurement above.

Matched has more interior structure, and local glyph orientation responds to image
features without hand-coded angle-to-glyph rules. Yet some planes still contain
bracket/ampersand runs. It does not recover the reference's richness from a simple
room just by examining more pixels. Doorway and crate boundaries remain clearer in
fast; matched's higher occupancy can mask them. Stair forms survive, but have more
busy interior marks. The enriched control is too sparse in several dark wall and
crate faces. Neither experimentally enhanced candidate is an unqualified improvement.

The exterior test has only ~11% occupied cells in both modes because the scene
actually has void around it; there is no screen-wide punctuation in uncovered cells.
The principal interior views genuinely have geometry almost everywhere. No fixed
occupancy goal was used, and no renderer was selected on a density score alone.

## Foreground intensity experiment

`scaled` solves an optimal clamped foreground multiplier for each glyph. `offset`
also solves a background offset as an image-space diagnostic. They are not offered
as interactive game backends. Explicit `--emit-intensity 1` with `scaled` or `gradient` writes
`grayscale.ansi` (equal R/G/B foreground values, no background fills) and `dim.ansi`
(normal/dim attributes), plus an image reconstruction. Nothing displays those
escape sequences automatically. No colors were added to the normal presenter.

[Dark/light comparison](visual/research/intensity/doorway/comparison.png) shows
normal, an approximate dim interpretation, and fixed grayscale. Actual dim behavior
is terminal-dependent. Most patches select dense glyphs such as `@`; the grayscale
version encodes the scene mostly through foreground brightness rather than shape.
Increasing exposure by 2.2 was also tested (`intensity_exposure/`): it brightens the
image without fixing the monotonous glyph vocabulary. Fixed grayscale also fails
to adapt naturally to light terminal backgrounds.

A separate, properly calibrated regular-versus-bold DejaVu trial is captured in
[font weight](visual/research/comparisons/font-weight.png). Bold produces heavier
stroke coverage, but still leaves repeated `x` fields and no decisive macro-shape
improvement. Changing terminal bold without changing the atlas would also introduce
a shape mismatch. Neither bold nor grayscale was added to the normal presenter.

The default-exposure doorway's scaled reconstruction MSE is .00950 versus .09049
when the **same selected glyphs** are displayed at full intensity. That is a real
numerical improvement but not a convincing stylistic win. Offset fitting has even
lower error on flat sources by reproducing the image in background fills, which
violates the intended display constraint.

Finally, a controlled comparison holds the **gradient matcher's selected glyphs
identical** and fits foreground alpha afterward. This avoids confusing intensity
with the joint optimizer's preference for dense glyphs. Inspect
[doorway](visual/research/matched_intensity/doorway/comparison.png),
[crates](visual/research/matched_intensity/crates/comparison.png), and
[stairs](visual/research/matched_intensity/stairs/comparison.png), each showing normal,
approximate dim, and fixed grayscale on dark/light themes. Occupancy and glyph
entropy remain exactly unchanged. Float-image MSE drops from .03225 to .01412,
.03667 to .01582, and .03463 to .01455 respectively (56–58%). Intensity fitting is
outside the timed loop; these numbers do not measure an interactive intensity mode.

On dark previews, intensity suppresses weak wall punctuation and improves plane
contrast. It does not fix the floor's repetitive pattern or establish reference-level
silhouette fidelity. Fixed grayscale encodes the wrong ink relationship on a light
background; dim's real appearance is terminal-dependent. Thus intensity is a useful
future opt-in display experiment, but not a theme-native replacement or sufficient
reason to adopt matched for gameplay.

## Architecture and exact commands

The existing scene, projection, clipping, rasterizer, depth buffer, collision,
controls, hot reload, accepted ASCII mapper and hybrid resolver are untouched.
`src/research/` is opt-in:

- `atlas.rs`: original bitmap strokes, area sampling, measured density, variance,
  gradient vectors and compact features; optional P5 font calibration loader.
- `source.rs`: inverse-project depth, estimate surface normals, world-space material
  variation and experimental lighting/contact/context passes.
- `matcher.rs`: competing objectives, bounded/coarse candidate search, stable ties,
  experimental Top-K/coherence and guarded cell-history hysteresis.
- `backend.rs`: supersampled source adapter for interactive `matched` and the
  `enriched` control. Persistent buffers; no per-cell heap allocations.
- `src/bin/research.rs`: controlled headless capture, staged timing, movement paths,
  reconstruction, distributions and optional intensity diagnostics.

Physical viewport aspect is preserved: the supersampled pixel ratio is terminal
cell ratio × sample height / sample width. Source patches and glyph templates
therefore cover the same physical cell shape. The default built-in atlas works with
no font installation. Optional font calibration runs offline via ImageMagick against
a user-selected local font. No TTF/OTF files or release font dependency are bundled.
Generated full calibration files are local/ignored; font-derived debug captures
remain as evidence, with the installed DejaVu license notice. The terminal's actual
font/rasterizer may still differ in stroke weight, antialiasing and line spacing.

```sh
cargo build --release --locked

# Accepted baseline remains the default; hybrid still works.
target/release/main --ascii-backend fast --glyph-set ascii
target/release/main --ascii-backend fast --glyph-set hybrid

# Practical matcher and source-only control.
target/release/main --ascii-backend matched --telemetry
target/release/main --ascii-backend matched --match-samples 2x4 --telemetry
target/release/main --ascii-backend enriched --telemetry

# Reproduce all three matched camera positions interactively.
target/release/main --ascii-backend matched --camera-preset doorway
target/release/main --ascii-backend matched --camera-preset crates
target/release/main --ascii-backend matched --camera-preset stairs

# Deterministic text and capture, no prior history.
target/release/main --ascii-backend matched --camera-preset doorway --snapshot --width 120 --height 40 > /tmp/matched-doorway.txt
target/release/main --ascii-backend matched --camera-preset doorway --capture-frame /tmp/matched-doorway --width 120 --height 40
# Replace doorway with crates/stairs and matched with fast/enriched for each A/B.

# Exact headless candidate with separated stage timings and reconstruction metrics.
target/release/research --method gradient --sample-width 3 --sample-height 6 --noise .22 --lighting 1 --black .055 --white .7 --hysteresis .035 --view doorway --frames 120 --motion yaw --output /tmp/matched-benchmark
# Add --width 200 --height 60 or --width 240 --height 80 for larger terminals.

# Optional font calibration; choose a font you actually use and have locally.
uv run --no-project python scripts/research_font.py /path/to/monospace.ttf /tmp/my-atlas.pgm 3 6
target/release/main --ascii-backend matched --matched-atlas /tmp/my-atlas.pgm

# Reproduce the full serial CPU and PTY performance tables.
uv run --no-project python scripts/research_benchmark.py
uv run --no-project python scripts/research_pty.py

# One full temporal sequence; four paths are yaw/forward/strafe/stairs.
target/release/research --method gradient --sample-width 3 --sample-height 6 --noise .22 --lighting 1 --black .055 --white .7 --hysteresis .035 --frames 180 --motion stairs --sequence 1 --output /tmp/matched-stairs-motion

# Optional intensity capture; inspect the generated files deliberately.
target/release/research --method scaled --sample-width 3 --sample-height 6 --noise .22 --lighting 1 --black .055 --white .7 --emit-intensity 1 --output /tmp/intensity
# Hold gradient-selected glyphs fixed and fit alpha afterward instead.
target/release/research --method gradient --sample-width 3 --sample-height 6 --noise .22 --lighting 1 --black .055 --white .7 --hysteresis .035 --emit-intensity 1 --view doorway --output /tmp/matched-intensity
```

Existing navigation, debug views 1–5, resize and exit behavior remain available.
Fast and enriched retain V/P/R visual tuning. Matched uses its documented research
settings: V is inactive and explicit fast-only preset/ramp/tone options are rejected
rather than silently ignored. Use the research runner to compare alternate matcher
and source settings. `frame.txt` contains the selected output vocabulary;
`ascii.txt` remains its ASCII intermediate. Matched captures additionally contain
`source.pgm`, `atlas.pgm`, `reconstruction.pgm` and `backend.txt`. Matched does not
write the accepted mapper's unused style/edge diagnostics.

## Validation, limitations and recommendation

50 tests pass, including every pre-existing correctness and ASCII/hybrid regression
check. New tests cover atlas density/features, template recovery, deterministic
matching, exact search bounds, fixed-glyph intensity recovery, blank coverage,
hysteresis reset/disocclusion,
unprojection, world-noise continuity, rejection of false planar contact shadows,
supersampling aspect, optional atlas loading, six finalist snapshots and CLI safety.
Formatting, strict Clippy, tests and cargo check pass. Visual success is assessed
separately above; passing these tests does not make matched the aesthetic winner.

Known limitations:

- Atlas coverage is not the same as actual terminal ink; optional calibration helps
  but does not control the user's font, antialiasing, bold/dim or line spacing.
- Procedural grayscale variation has no semantic material identity. The source still
  lacks the meaningful surface detail of the reference's original 3D project.
- Reconstructed normals can be unreliable at depth discontinuities. This is an
  isolated screen-space source experiment, not a new material/normal-buffer system.
- Cell-history hysteresis reduces popping but is not motion compensation. Fine
  details may stick before changing; strafe/stair views remain noticeably busier.
- More samples cannot create detail missing from the source. At 8×16, cost is well
  outside the real-time target without a decisive visual improvement.
- The reference's algorithm and original font are unknown. Its compression and
  different scene prevent pixel-registered similarity or confident glyph counts.
- The grayscale/intensity previews are diagnostic models; no interactive grayscale
  FPS or real-terminal dim appearance is claimed.

**Final recommendation:** keep fast as the gameplay/default renderer and preserve
matched as an experimental review mode. The best practical matched method is
normalized shape plus gradient matching with measured atlas density, meaningful
source variation, exact bounded search and modest hysteresis. It wins local texture
interest, but has not demonstrated an overall readability/motion/performance win.
The source renderer still limits fidelity. Richer, purposeful material and lighting
information is more promising than another glyph-family or screen-noise pass.

The next rendering research step should use a small, controlled grayscale material
fixture with calibrated font masks and motion-aware stability checks, while reducing
source-pass cost (potentially retaining surface normals/positions rather than
reconstructing them). Re-test against fast on the same input before adopting a new
backend for gameplay. General mesh import and networking remain out of this task.

## Changed files and evidence

Runtime integration is limited to `src/app.rs` and `src/lib.rs`. New isolated code is
`src/research/{mod,atlas,matcher,source,backend}.rs` and `src/bin/research.rs`; new
regression coverage is `tests/research_regression.rs`. No dependencies were added.

Research tooling is `scripts/research_{compare,font,intensity,motion,benchmark,pty,summary}.py`.
Documentation is this report and a link from `BUILD.md`. `.gitignore` excludes
regenerable font atlases and unpacked motion frames; `.gitattributes` permits the
intentional terminal control payload in diagnostic `.ansi` files.

`visual/research/` retains the reference, baseline, original atlas preview, failed
trials, source/reconstruction images, final captures, font license notice, performance
records and experiment index. Motion frames are stored losslessly in `temporal/review.html`
instead of committing thousands of separate text files. Each later experiment's
`command.txt` records its invocation; historical source-v1 limitations are noted above.

Final validation also passed the original ASCII/hybrid PTY input, resize, exit,
width-rejection and restoration checks, plus hot-reload ABI rejection/replacement/
state-continuity checks. Experimental PTY results are in `performance/pty.json`.
No CI/README standards changes were made: this VinnovateIT repository is excluded
from that rollout by the workspace policy.
