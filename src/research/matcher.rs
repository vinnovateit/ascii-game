use super::atlas::{Atlas, Features, Glyph, MAX_PIXELS};
use crate::render::framebuffer::Framebuffer;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    Mse,
    Normalized,
    Gradient,
    Feature,
    Scaled,
    Offset,
}
impl Method {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "mse" => Self::Mse,
            "normalized" => Self::Normalized,
            "gradient" => Self::Gradient,
            "feature" => Self::Feature,
            "scaled" => Self::Scaled,
            "offset" => Self::Offset,
            _ => return None,
        })
    }
}
#[derive(Clone)]
pub struct Settings {
    pub method: Method,
    pub shortlist: usize,
    pub top_k: usize,
    pub tolerance: f32,
    pub hysteresis: f32,
    pub coherence: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Gradient,
            shortlist: 0,
            top_k: 1,
            tolerance: 0.025,
            hysteresis: 0.,
            coherence: 0.,
        }
    }
}
pub struct Matcher {
    pub atlas: Atlas,
    pub settings: Settings,
    pub alphas: Vec<f32>,
    pub offsets: Vec<f32>,
    previous: Vec<usize>,
    means: Vec<f32>,
    depths: Vec<f32>,
    dimensions: (usize, usize),
}
impl Matcher {
    pub fn new(atlas: Atlas, settings: Settings) -> Self {
        Self {
            atlas,
            settings,
            alphas: Vec::new(),
            offsets: Vec::new(),
            previous: Vec::new(),
            means: Vec::new(),
            depths: Vec::new(),
            dimensions: (0, 0),
        }
    }
    pub fn reset(&mut self) {
        self.previous.clear();
        self.means.clear();
        self.depths.clear();
    }
    pub fn convert(&mut self, fb: &Framebuffer, cells: &mut [u8]) {
        let w = self.atlas.width;
        let h = self.atlas.height;
        let n = w * h;
        let cols = fb.width / w;
        let rows = fb.height / h;
        assert_eq!(cols * rows, cells.len());
        if self.dimensions != (cols, rows) {
            self.reset();
            self.dimensions = (cols, rows);
        }
        if self.previous.len() != cells.len() {
            self.previous = vec![usize::MAX; cells.len()];
            self.means = vec![0.; cells.len()];
            self.depths = vec![f32::INFINITY; cells.len()];
        }
        self.alphas.resize(cells.len(), 1.);
        self.offsets.resize(cells.len(), 0.);
        let mut patch = [0.; MAX_PIXELS];
        for cy in 0..rows {
            for cx in 0..cols {
                let i = cy * cols + cx;
                let mut depth = f32::INFINITY;
                for y in 0..h {
                    for x in 0..w {
                        let j = (cy * h + y) * fb.width + cx * w + x;
                        patch[y * w + x] = fb.luminance[j];
                        depth = depth.min(fb.depth[j]);
                    }
                }
                if !depth.is_finite() {
                    cells[i] = b' ';
                    self.previous[i] = 0;
                    self.means[i] = 0.;
                    self.depths[i] = depth;
                    self.alphas[i] = 1.;
                    self.offsets[i] = 0.;
                    continue;
                }
                let p = &patch[..n];
                let f = Features::extract(p, w, h);
                let mut candidates = [(f32::INFINITY, 0usize); 64];
                let count = self.atlas.glyphs.len();
                let bounded = self.settings.shortlist == 0
                    && matches!(self.settings.method, Method::Normalized | Method::Gradient);
                for (g, slot) in self
                    .atlas
                    .glyphs
                    .iter()
                    .zip(candidates.iter_mut())
                    .enumerate()
                {
                    let (glyph, slot) = slot;
                    *slot = (
                        if bounded {
                            8. * ((f.mean - glyph.features.mean) / self.atlas.max_density).powi(2)
                        } else {
                            coarse(&f, &glyph.features, self.atlas.max_density)
                        },
                        g,
                    );
                }
                let candidate_count = if self.settings.shortlist > 0 {
                    self.settings.shortlist.min(count)
                } else {
                    count
                };
                if candidate_count < count {
                    candidates[..count]
                        .sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                }
                if bounded {
                    let nearest = candidates[..count]
                        .iter()
                        .enumerate()
                        .min_by(|a, b| a.1.0.total_cmp(&b.1.0))
                        .unwrap()
                        .0;
                    candidates.swap(0, nearest);
                }
                let mut ranked = [(f32::INFINITY, 0usize, 1., 0.); 64];
                let mut evaluated = 0;
                let mut best_score = f32::INFINITY;
                for &(bound, g) in &candidates[..candidate_count] {
                    if bounded && bound > best_score + self.settings.tolerance + 1e-6 {
                        continue;
                    }

                    let (mut score, alpha, beta) = score(
                        p,
                        &f,
                        &self.atlas.glyphs[g],
                        self.settings.method,
                        self.atlas.max_density,
                    );
                    if cx > 0
                        && (f.mean - self.means[i - 1]).abs() < 0.025
                        && self.previous[i - 1] != g
                    {
                        score += self.settings.coherence;
                    }
                    ranked[evaluated] = (score, g, alpha, beta);
                    evaluated += 1;
                    best_score = best_score.min(score);
                }
                ranked[..evaluated]
                    .sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                let k = self.settings.top_k.min(evaluated).max(1);
                let near = ranked[..k]
                    .iter()
                    .take_while(|v| v.0 <= ranked[0].0 + self.settings.tolerance)
                    .count();
                // Variation follows quantized patch content, never screen coordinates or time.
                let fingerprint = p.iter().fold(2166136261u32, |h, v| {
                    (h ^ ((*v * 31.).round() as u32)).wrapping_mul(16777619)
                });
                let mut best = ranked[fingerprint as usize % near];
                let old = self.previous[i];
                if self.settings.hysteresis > 0.
                    && old < count
                    && (self.depths[i] - depth).abs() < 0.0005
                    && (self.means[i] - f.mean).abs() < 0.04
                {
                    let (s, a, b) = score(
                        p,
                        &f,
                        &self.atlas.glyphs[old],
                        self.settings.method,
                        self.atlas.max_density,
                    );
                    if s <= best.0 + self.settings.hysteresis {
                        best = (s, old, a, b);
                    }
                }
                cells[i] = self.atlas.glyphs[best.1].byte;
                self.previous[i] = best.1;
                self.means[i] = f.mean;
                self.depths[i] = depth;
                self.alphas[i] = best.2;
                self.offsets[i] = best.3;
            }
        }
    }
    /// Diagnostic only: fit foreground intensity without changing the selected glyphs.
    pub fn fit_intensity(&mut self, fb: &Framebuffer, cells: &[u8]) {
        let w = self.atlas.width;
        let h = self.atlas.height;
        let cols = fb.width / w;
        assert_eq!(cells.len(), cols * (fb.height / h));
        self.alphas.resize(cells.len(), 1.);
        self.offsets.resize(cells.len(), 0.);
        for (i, c) in cells.iter().enumerate() {
            let g = self.atlas.glyphs.iter().find(|g| g.byte == *c).unwrap();
            let mut dot = 0.;
            let mut energy = 0.;
            for y in 0..h {
                for x in 0..w {
                    let mask = g.mask[y * w + x];
                    let p = fb.luminance[(i / cols * h + y) * fb.width + i % cols * w + x];
                    dot += p * mask;
                    energy += mask * mask;
                }
            }
            self.alphas[i] = (dot / energy.max(1e-8)).clamp(0., 1.);
            self.offsets[i] = 0.;
        }
    }
    pub fn reconstruct(&self, cells: &[u8], cols: usize, intensity: bool) -> Vec<f32> {
        let w = self.atlas.width;
        let h = self.atlas.height;
        let mut result = vec![0.; cells.len() * w * h];
        for (i, c) in cells.iter().enumerate() {
            if let Some(g) = self.atlas.glyphs.iter().find(|g| g.byte == *c) {
                for y in 0..h {
                    for x in 0..w {
                        result[(i / cols * h + y) * cols * w + i % cols * w + x] = if intensity {
                            g.mask[y * w + x] * self.alphas[i] + self.offsets[i]
                        } else {
                            g.mask[y * w + x]
                        };
                    }
                }
            }
        }
        result
    }
}
fn coarse(a: &Features, b: &Features, max: f32) -> f32 {
    let shape = a
        .small
        .iter()
        .zip(b.small)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>();
    8. * ((a.mean - b.mean) / max).powi(2) + 0.18 * shape * (a.variance.sqrt() / 0.06).min(1.)
}
fn score(p: &[f32], f: &Features, g: &Glyph, method: Method, max: f32) -> (f32, f32, f32) {
    let n = p.len() as f32;
    if method == Method::Feature {
        return (coarse(f, &g.features, max), 1., 0.);
    }
    let dot = p.iter().zip(&g.mask).map(|(p, g)| p * g).sum::<f32>() / n;
    let g2 = g.features.variance + g.features.mean.powi(2);
    let p2 = f.variance + f.mean.powi(2);
    if matches!(method, Method::Mse | Method::Scaled | Method::Offset) {
        let (alpha, beta) = match method {
            Method::Scaled => ((dot / (g2 + 1e-8)).clamp(0., 1.), 0.),
            Method::Offset => {
                let a =
                    ((dot - f.mean * g.features.mean) / (g.features.variance + 1e-8)).clamp(0., 1.);
                (a, (f.mean - a * g.features.mean).clamp(0., 1. - a))
            }
            _ => (1., 0.),
        };
        return (
            p2 + alpha * alpha * g2 + beta * beta - 2. * alpha * dot - 2. * beta * f.mean
                + 2. * alpha * beta * g.features.mean,
            alpha,
            beta,
        );
    }
    let corr = ((dot - f.mean * g.features.mean)
        / (f.variance * g.features.variance + 1e-10).sqrt())
    .clamp(-1., 1.);
    let reliability = (f.variance.sqrt() / 0.07).min(1.);
    let luma = 8. * ((f.mean - g.features.mean) / max).powi(2);
    let shape = reliability * (1. - corr) * 0.7;
    let grad = if method == Method::Gradient {
        let cross = f.dx[..p.len()]
            .iter()
            .zip(&g.features.dx)
            .zip(f.dy.iter().zip(&g.features.dy))
            .map(|((a, b), (c, d))| a * b + c * d)
            .sum::<f32>()
            / n;
        reliability
            * (1. - (cross / (f.energy * g.features.energy + 1e-10).sqrt()).clamp(-1., 1.))
            * 0.3
    } else {
        0.
    };
    (luma + shape + grad, 1., 0.)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intensity_fit_preserves_glyph_and_recovers_scaled_mask() {
        let atlas = Atlas::builtin(3, 6, false);
        let glyph = atlas.glyphs.iter().find(|g| g.byte == b'X').unwrap();
        let mut fb = Framebuffer::new(3, 6);
        fb.luminance = glyph.mask.iter().map(|v| v * 0.4).collect();
        let mut matcher = Matcher::new(atlas, Settings::default());
        let cells = *b"X";
        matcher.fit_intensity(&fb, &cells);
        assert_eq!(cells, *b"X");
        assert!((matcher.alphas[0] - 0.4).abs() < 1e-6);
        assert_eq!(matcher.offsets, [0.]);
        let reconstruction = matcher.reconstruct(&cells, 1, true);
        assert!(
            reconstruction
                .iter()
                .zip(fb.luminance)
                .all(|(a, b)| (a - b).abs() < 1e-6)
        );
    }
    #[test]
    fn density_bound_preserves_full_search_winner_and_top_k() {
        let mut fb = Framebuffer::new(6 * 64, 10);
        fb.depth.fill(0.9);
        for (i, l) in fb.luminance.iter_mut().enumerate() {
            *l = (crate::render::ascii::hash(i % 384, i / 384) & 65535) as f32 / 65535. * 0.4;
        }
        for method in [Method::Normalized, Method::Gradient] {
            for top_k in [1, 3] {
                let mut bounded = Matcher::new(
                    Atlas::builtin(6, 10, false),
                    Settings {
                        method,
                        top_k,
                        ..Settings::default()
                    },
                );
                let mut full = Matcher::new(
                    Atlas::builtin(6, 10, false),
                    Settings {
                        method,
                        top_k,
                        shortlist: 64,
                        ..Settings::default()
                    },
                );
                let mut a = vec![b' '; 64];
                let mut b = a.clone();
                bounded.convert(&fb, &mut a);
                full.convert(&fb, &mut b);
                assert_eq!(a, b);
            }
        }
    }
    #[test]
    fn exact_template_is_recovered_by_mse() {
        let atlas = Atlas::builtin(6, 10, false);
        for glyph in &atlas.glyphs {
            let f = &glyph.features;
            let own = score(&glyph.mask, f, glyph, Method::Mse, atlas.max_density).0;
            assert!(own.abs() < 1e-6);
            for other in &atlas.glyphs {
                assert!(
                    score(&glyph.mask, f, other, Method::Mse, atlas.max_density).0 >= own - 1e-6
                );
            }
        }
    }
    #[test]
    fn empty_coverage_is_space_even_with_bright_luminance() {
        let mut fb = Framebuffer::new(6, 10);
        fb.luminance.fill(1.);
        let mut m = Matcher::new(Atlas::builtin(6, 10, false), Settings::default());
        let mut c = *b"X";
        m.convert(&fb, &mut c);
        assert_eq!(c, [b' ']);
    }
    #[test]
    fn deterministic_patch_and_hysteresis_reset() {
        let mut fb = Framebuffer::new(6, 10);
        fb.depth.fill(0.9);
        fb.luminance.fill(0.2);
        let mut m = Matcher::new(
            Atlas::builtin(6, 10, false),
            Settings {
                top_k: 3,
                hysteresis: 0.03,
                ..Settings::default()
            },
        );
        let mut c = *b" ";
        m.convert(&fb, &mut c);
        let first = c;
        m.convert(&fb, &mut c);
        assert_eq!(c, first);
        m.reset();
        m.convert(&fb, &mut c);
        assert_eq!(c, first);
    }
}
