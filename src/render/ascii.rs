pub use super::style::AsciiStyle;
use super::{capture::linear_depth, framebuffer::Framebuffer};
use crate::engine::camera::Camera;
pub struct Ascii {
    pub style: AsciiStyle,
    pub bands: Vec<Vec<u8>>,
    intensity: Vec<f32>,
    edges: Vec<Edge>,
}
impl Default for Ascii {
    fn default() -> Self {
        Self {
            style: AsciiStyle::default(),
            intensity: Vec::new(),
            edges: Vec::new(),
            bands: vec![
                b" ".to_vec(),
                b".'".to_vec(),
                b":;".to_vec(),
                b"xX".to_vec(),
                b"#%".to_vec(),
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
    fn mapped(&self, l: f32, x: usize, y: usize, variation: f32) -> u8 {
        const BAYER: [[f32; 4]; 4] = [
            [0., 8., 2., 10.],
            [12., 4., 14., 6.],
            [3., 11., 1., 9.],
            [15., 7., 13., 5.],
        ];
        // This hard gate precedes dithering: negative space cannot be refilled.
        if l < self.style.background_cutoff || l <= 0. || self.bands.is_empty() {
            return b' ';
        }
        let levels = self.style.posterization_levels.max(2) as f32 - 1.;
        let q = ((l * levels + (BAYER[y % 4][x % 4] / 16. - 0.46875) * self.style.dither_strength)
            .round()
            / levels)
            .clamp(0., 1.);
        let index = (q * (self.bands.len() - 1) as f32).round() as usize;
        let band = &self.bands[index];
        if band.is_empty() {
            return b' ';
        }
        let h = hash(x, y);
        // Low-density bands stay canonical; variation is opt-in and spatially stable.
        let variant =
            if q >= 0.4 && (h & 65535) as f32 / 65535. < self.style.texture_variance * variation {
                (h >> 16) as usize % band.len()
            } else {
                0
            };
        band[variant]
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
        for i in 0..self.intensity.len() {
            let depth = linear_depth(fb.depth[i], camera);
            self.intensity[i] = if depth.is_finite() {
                self.tone(fb.luminance[i]) / (1. + (self.style.depth_falloff * depth).powf(1.3))
            } else {
                0.
            };
        }
        for y in 0..fb.height {
            for x in 0..fb.width {
                let i = y * fb.width + x;
                let edge = gradient(&self.intensity, fb.width, fb.height, x, y);
                self.edges[i] = edge;
                let l = self.intensity[i];
                // Flat surfaces only get a small fraction of the allowed texture;
                // strong silhouette gradients suppress variants completely.
                let variation = if edge.magnitude > self.style.edge_threshold {
                    0.
                } else {
                    (0.3 + edge.magnitude * 6.).min(1.)
                };
                let mut glyph = self.mapped(l, x, y, variation);
                if glyph != b' '
                    && edge.magnitude * self.style.edge_weight > self.style.edge_threshold
                {
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
#[derive(Clone, Copy, Default)]
struct Edge {
    gx: f32,
    gy: f32,
    magnitude: f32,
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
    Edge {
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
        assert_eq!(a.glyph(0., 2, 2), b' ');
        let first: Vec<_> = (0..64).map(|x| a.glyph(0.7, x, 7)).collect();
        assert_eq!(
            first,
            (0..64).map(|x| a.glyph(0.7, x, 7)).collect::<Vec<_>>()
        );
        assert!(first.windows(2).any(|p| p[0] != p[1]));
        assert_ne!(hash(3, 5), hash(5, 3));
    }
    #[test]
    fn cutoff_cannot_be_refilled_by_dither() {
        let mut a = Ascii::default();
        a.style.dither_strength = 2.;
        for x in 0..16 {
            for y in 0..16 {
                assert_eq!(a.mapped(a.style.background_cutoff - 0.001, x, y, 1.), b' ');
            }
        }
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
