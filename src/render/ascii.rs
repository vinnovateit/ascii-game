pub use super::style::AsciiStyle;
use super::{capture::linear_depth, framebuffer::Framebuffer};
use crate::engine::camera::Camera;
pub struct Ascii {
    pub style: AsciiStyle,
    pub bands: Vec<Vec<u8>>,
    intensity: Vec<f32>,
    edges: Vec<Edge>,
    depth_signal: Vec<f32>,
}
impl Default for Ascii {
    fn default() -> Self {
        Self {
            style: AsciiStyle::default(),
            intensity: Vec::new(),
            edges: Vec::new(),
            depth_signal: Vec::new(),
            bands: vec![
                b" ".to_vec(),
                b".,'`".to_vec(),
                b":;-".to_vec(),
                b"=+~".to_vec(),
                b"x*X".to_vec(),
                b"#%&".to_vec(),
                b"MW@".to_vec(),
            ],
        }
    }
}
pub fn hash(x: usize, y: usize) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9e3779b9) ^ (y as u32).wrapping_mul(0x85ebca6b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    h
}
impl Ascii {
    pub fn set_ramp(&mut self, ramp: &str) -> Result<(), String> {
        if ramp.is_empty() || !ramp.bytes().all(|b| (32..=126).contains(&b)) {
            return Err("ramp must contain printable single-column ASCII".into());
        }
        self.bands = ramp.bytes().map(|b| vec![b]).collect();
        Ok(())
    }
    pub fn tone(&self, l: f32) -> f32 {
        let s = &self.style;
        let normalized =
            ((l * s.exposure - s.black_point) / (s.white_point - s.black_point)).clamp(0., 1.);
        ((normalized.powf(1. / s.gamma) - 0.5) * s.contrast + 0.5).clamp(0., 1.)
    }
    /// Map a COVERED sample. Empty background is rejected by convert using depth.
    fn mapped(&self, l: f32, x: usize, y: usize, variation: f32) -> u8 {
        const BAYER: [[f32; 4]; 4] = [
            [0., 8., 2., 10.],
            [12., 4., 14., 6.],
            [3., 11., 1., 9.],
            [15., 7., 13., 5.],
        ];
        if self.bands.is_empty() {
            return b' ';
        }
        let h = hash(x, y);
        let noise = (h & 65535) as f32 / 65535.;
        // Cutoff is now a shadow transition, NOT a geometry rejection threshold.
        // Even a fully black covered surface gets sparse, stable marks.
        if l < self.style.shadow_threshold {
            let presence = 0.10 + 0.80 * (l / self.style.shadow_threshold.max(0.001));
            if noise > presence {
                return b' ';
            }
            let band = &self.bands[1.min(self.bands.len() - 1)];
            return variant(band, h.rotate_left(11), self.style.texture_variance);
        }
        let levels = self.style.posterization_levels.max(2) as f32 - 1.;
        let amount = self.style.texture_variance * variation;
        let micro = (noise - 0.5) * 0.12 * amount;
        let q = ((l + micro) * levels
            + (BAYER[y % 4][x % 4] / 16. - 0.46875) * self.style.dither_strength)
            .round()
            / levels;
        let band_position = q.clamp(0., 1.) * (self.bands.len() - 1) as f32;
        // Blend adjacent bands spatially instead of making whole planes a single glyph.
        let low = band_position.floor() as usize;
        let fraction = band_position.fract();
        let choose = (hash(x.wrapping_add(733), y.wrapping_add(977)) & 65535) as f32 / 65535.;
        let threshold = 0.5 + (choose - 0.5) * amount;
        let index = (low + usize::from(fraction > threshold))
            .min(self.bands.len() - 1)
            .max(1.min(self.bands.len() - 1));
        variant(&self.bands[index], h.rotate_left(11), amount)
    }
    pub fn glyph(&self, l: f32, x: usize, y: usize) -> u8 {
        self.mapped(self.tone(l), x, y, 1.)
    }
    pub fn convert(&mut self, fb: &Framebuffer, camera: &Camera, cells: &mut [u8], plain: bool) {
        if plain {
            for (l, cell) in fb.luminance.iter().zip(cells) {
                *cell = b" .:-=+*#%@"[(l.clamp(0., 1.) * 9.).round() as usize];
            }
            return;
        }
        self.intensity.resize(fb.luminance.len(), 0.);
        self.edges.resize(fb.luminance.len(), Edge::default());
        self.depth_signal.resize(fb.luminance.len(), 0.);
        for i in 0..self.intensity.len() {
            let depth = linear_depth(fb.depth[i], camera);
            self.intensity[i] = if covered(fb.depth[i]) {
                // Soft attenuation has a floor: distance cannot erase real surfaces.
                let attenuation = 0.4 + 0.6 / (1. + (self.style.depth_falloff * depth).powf(1.3));
                self.tone(fb.luminance[i]) * attenuation
            } else {
                0.
            };
            self.depth_signal[i] = if covered(fb.depth[i]) {
                depth.ln_1p()
            } else {
                camera.far.ln_1p()
            };
        }
        for y in 0..fb.height {
            for x in 0..fb.width {
                let i = y * fb.width + x;
                if !covered(fb.depth[i]) {
                    cells[i] = b' ';
                    self.edges[i] = Edge::default();
                    continue;
                }
                let luminance_edge = gradient(&self.intensity, fb.width, fb.height, x, y);
                let mut depth_edge = gradient(&self.depth_signal, fb.width, fb.height, x, y);
                depth_edge.magnitude = (depth_edge.magnitude * 0.55).min(1.);
                let mut edge = if depth_edge.magnitude > luminance_edge.magnitude {
                    depth_edge
                } else {
                    luminance_edge
                };
                edge.deviation = luminance_edge.deviation;
                self.edges[i] = edge;
            }
        }
        for y in 0..fb.height {
            for x in 0..fb.width {
                let i = y * fb.width + x;
                if !covered(fb.depth[i]) {
                    continue;
                }
                let edge = self.edges[i];
                // Thin Sobel ridges before substituting glyphs. A consistent tie
                // rule chooses only one side of a two-cell gradient plateau.
                let (previous, next) = if edge.gx.abs() > edge.gy.abs() {
                    (
                        y * fb.width + x.saturating_sub(1),
                        y * fb.width + (x + 1).min(fb.width - 1),
                    )
                } else {
                    (
                        y.saturating_sub(1) * fb.width + x,
                        (y + 1).min(fb.height - 1) * fb.width + x,
                    )
                };
                let ridge = edge.magnitude > self.edges[previous].magnitude + 1e-5
                    && edge.magnitude >= self.edges[next].magnitude - 1e-5;
                let boundary =
                    ridge && edge.magnitude * self.style.edge_weight > self.style.edge_threshold;
                let deviation = edge.deviation;
                let variation = if boundary {
                    0.3
                } else {
                    (0.85 + 1.5 * deviation).min(1.)
                };
                let mut glyph = self.mapped(self.intensity[i], x, y, variation);
                // Covered dark-side boundaries can be shown even when shadow stippling
                // chose a space. Uncovered cells never reach this branch.
                if boundary {
                    glyph = edge.glyph();
                }
                cells[i] = glyph;
            }
        }
    }
    pub fn edge_values(&self) -> impl Iterator<Item = f32> + '_ {
        self.edges.iter().map(|e| e.magnitude)
    }
}
fn covered(z: f32) -> bool {
    z.is_finite() && (0.0..=1.0).contains(&z)
}
fn variant(band: &[u8], h: u32, amount: f32) -> u8 {
    if band.is_empty() {
        return b' ';
    }
    if (h & 65535) as f32 / 65535. < amount {
        band[(h >> 16) as usize % band.len()]
    } else {
        band[0]
    }
}
#[derive(Clone, Copy, Default)]
struct Edge {
    gx: f32,
    gy: f32,
    magnitude: f32,
    deviation: f32,
}
impl Edge {
    fn glyph(self) -> u8 {
        if self.gx.abs() > self.gy.abs() * 2. {
            b'|'
        } else if self.gy.abs() > self.gx.abs() * 2. {
            b'='
        } else if self.gx * self.gy > 0. {
            b'/'
        } else {
            b'\\'
        }
    }
}
/// 3x3 Sobel gradient; clamped neighbors avoid spurious borders and handle 1x1.
fn gradient(values: &[f32], width: usize, height: usize, x: usize, y: usize) -> Edge {
    let left = x.saturating_sub(1);
    let right = (x + 1).min(width - 1);
    let top = y.saturating_sub(1);
    let bottom = (y + 1).min(height - 1);
    let a = values[top * width + left];
    let b = values[top * width + x];
    let c = values[top * width + right];
    let d = values[y * width + left];
    let f = values[y * width + right];
    let g = values[bottom * width + left];
    let h = values[bottom * width + x];
    let i = values[bottom * width + right];
    let gx = (c + 2. * f + i - a - 2. * d - g) * 0.25;
    let gy = (g + 2. * h + i - a - 2. * b - c) * 0.25;
    let samples = [a, b, c, d, values[y * width + x], f, g, h, i];
    let mean = samples.iter().sum::<f32>() / 9.;
    let variance = samples.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / 9.;
    Edge {
        deviation: variance.sqrt(),
        gx,
        gy,
        magnitude: gx.hypot(gy).min(1.),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_variants_and_bands() {
        let a = Ascii::default();

        let first: Vec<_> = (0..64).map(|x| a.glyph(0.7, x, 7)).collect();
        assert_eq!(
            first,
            (0..64).map(|x| a.glyph(0.7, x, 7)).collect::<Vec<_>>()
        );
        assert!(first.windows(2).any(|p| p[0] != p[1]));
        assert_ne!(hash(3, 5), hash(5, 3));
    }
    #[test]
    fn dark_covered_samples_keep_deterministic_presence() {
        let a = Ascii::default();
        let cells: Vec<_> = (0..256).map(|x| a.glyph(0., x, 2)).collect();
        assert!(cells.iter().any(|c| *c != b' '));
        assert!(cells.contains(&b' '));
        assert_eq!(
            cells,
            (0..256).map(|x| a.glyph(0., x, 2)).collect::<Vec<_>>()
        );
    }
    #[test]
    fn zero_variance_is_canonical() {
        let mut a = Ascii::default();
        a.style.texture_variance = 0.;
        a.style.dither_strength = 0.;
        let first = a.glyph(0.7, 0, 0);
        for x in 0..100 {
            assert_eq!(a.glyph(0.7, x, 0), first);
        }
    }
    #[test]
    fn ramp_rejects_terminal_controls_and_unicode() {
        let mut a = Ascii::default();
        assert!(a.set_ramp("\x1b").is_err());
        assert!(a.set_ramp("█").is_err());
        assert!(a.set_ramp(" .#").is_ok());
    }
}
