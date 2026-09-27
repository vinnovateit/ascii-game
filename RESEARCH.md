# Glyph reconstruction research log

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
No inference here establishes Pixatool's actual algorithm.

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
