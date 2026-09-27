use std::{fs, io, path::Path};
pub const MAX_PIXELS: usize = 128;
pub const CANDIDATES: &[u8] = b" .,'`:;!il|/\\-_~=+*xX&%#MW@<>^v[]()";
pub const COMPACT: &[u8] = b" .,:|/\\-=+x#M@";
#[derive(Clone)]
pub struct Features {
    pub mean: f32,
    pub variance: f32,
    pub energy: f32,
    pub small: [f32; 8],
    pub dx: [f32; MAX_PIXELS],
    pub dy: [f32; MAX_PIXELS],
}
impl Features {
    pub fn extract(p: &[f32], w: usize, h: usize) -> Self {
        let n = (w * h) as f32;
        let mean = p.iter().sum::<f32>() / n;
        let variance = p.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
        let mut f = Self {
            mean,
            variance,
            energy: 0.,
            small: [0.; 8],
            dx: [0.; MAX_PIXELS],
            dy: [0.; MAX_PIXELS],
        };
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let dx = (p[y * w + (x + 1).min(w - 1)] - p[y * w + x.saturating_sub(1)]) * 0.5;
                let dy = (p[(y + 1).min(h - 1) * w + x] - p[y.saturating_sub(1) * w + x]) * 0.5;
                f.dx[i] = dx;
                f.dy[i] = dy;
                f.energy += dx * dx + dy * dy;
                f.small[0] += dx.abs();
                f.small[1] += dy.abs();
                f.small[2] += (dx + dy).abs();
                f.small[3] += (dx - dy).abs();
                f.small[4 + (y * 2 / h) * 2 + x * 2 / w] += p[i] - mean;
            }
        }
        f.energy /= n;
        for v in &mut f.small {
            *v /= n * (variance.sqrt() + 0.04);
        }
        f
    }
}
pub struct Glyph {
    pub byte: u8,
    pub mask: Vec<f32>,
    pub features: Features,
}
pub struct Atlas {
    pub width: usize,
    pub height: usize,
    pub glyphs: Vec<Glyph>,
    pub max_density: f32,
}
impl Atlas {
    /// Original, deliberately simple 5×7 stroke drawings in a 6×10 cell.
    /// No font files or external font runtime are required.
    pub fn builtin(width: usize, height: usize, compact: bool) -> Self {
        assert!(width > 0 && height > 0 && width * height <= MAX_PIXELS);
        let mut glyphs = Vec::new();
        for &byte in if compact { COMPACT } else { CANDIDATES } {
            let rows = rows(byte);
            let mut mask = vec![0.; width * height];
            // Area sampling of the original bitmap; preserves measured ink coverage.
            for y in 0..height {
                for x in 0..width {
                    for sy in 0..4 {
                        for sx in 0..4 {
                            let u =
                                ((x as f32 + (sx as f32 + 0.5) / 4.) * 6. / width as f32) as usize;
                            let v = ((y as f32 + (sy as f32 + 0.5) / 4.) * 10. / height as f32)
                                as usize;
                            if u < 5 && (1..8).contains(&v) && rows[v - 1] & (1 << (4 - u)) != 0 {
                                mask[y * width + x] += 1. / 16.;
                            }
                        }
                    }
                }
            }
            let features = Features::extract(&mask, width, height);
            glyphs.push(Glyph {
                byte,
                mask,
                features,
            });
        }
        Self::from_glyphs(width, height, glyphs)
    }
    fn from_glyphs(width: usize, height: usize, glyphs: Vec<Glyph>) -> Self {
        let max_density = glyphs.iter().map(|g| g.features.mean).fold(0., f32::max);
        Self {
            width,
            height,
            glyphs,
            max_density,
        }
    }
    /// Optional calibration file: P5/8-bit, all 95 printable ASCII cells in one row.
    pub fn load(path: &Path, width: usize, height: usize, compact: bool) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let header = format!("P5\n{} {}\n255\n", width * 95, height);
        if !bytes.starts_with(header.as_bytes())
            || bytes.len() != header.len() + width * 95 * height
        {
            return Err(io::Error::other(
                "atlas must be canonical P5, 95 horizontal ASCII cells, 8-bit; use scripts/research_font.py",
            ));
        }
        let pixels = &bytes[header.len()..];
        let glyphs = if compact { COMPACT } else { CANDIDATES }
            .iter()
            .map(|&byte| {
                let mut mask = Vec::with_capacity(width * height);
                for y in 0..height {
                    for x in 0..width {
                        mask.push(
                            pixels[y * width * 95 + (byte - 32) as usize * width + x] as f32 / 255.,
                        );
                    }
                }
                let features = Features::extract(&mask, width, height);
                Glyph {
                    byte,
                    mask,
                    features,
                }
            })
            .collect();
        Ok(Self::from_glyphs(width, height, glyphs))
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let width = self.width * self.glyphs.len();
        crate::render::capture::pgm(
            path,
            width,
            self.height,
            (0..self.height).flat_map(|y| {
                self.glyphs
                    .iter()
                    .flat_map(move |g| g.mask[y * self.width..(y + 1) * self.width].iter().copied())
            }),
        )
    }
}
fn rows(c: u8) -> [u8; 7] {
    match c {
        b'.' => [0, 0, 0, 0, 0, 0, 4],
        b',' => [0, 0, 0, 0, 0, 4, 8],
        b'\'' => [4, 4, 8, 0, 0, 0, 0],
        b'`' => [8, 4, 0, 0, 0, 0, 0],
        b':' => [0, 4, 0, 0, 0, 4, 0],
        b';' => [0, 4, 0, 0, 4, 4, 8],
        b'!' => [4, 4, 4, 4, 4, 0, 4],
        b'i' => [4, 0, 12, 4, 4, 4, 14],
        b'l' => [12, 4, 4, 4, 4, 4, 14],
        b'|' => [4, 4, 4, 4, 4, 4, 4],
        b'/' => [1, 2, 2, 4, 8, 8, 16],
        b'\\' => [16, 8, 8, 4, 2, 2, 1],
        b'-' => [0, 0, 0, 31, 0, 0, 0],
        b'_' => [0, 0, 0, 0, 0, 0, 31],
        b'~' => [0, 0, 9, 22, 0, 0, 0],
        b'=' => [0, 0, 31, 0, 31, 0, 0],
        b'+' => [0, 4, 4, 31, 4, 4, 0],
        b'*' => [0, 21, 14, 31, 14, 21, 0],
        b'x' => [0, 0, 17, 10, 4, 10, 17],
        b'X' => [17, 17, 10, 4, 10, 17, 17],
        b'&' => [12, 18, 20, 8, 21, 18, 13],
        b'%' => [25, 26, 2, 4, 8, 11, 19],
        b'#' => [10, 31, 10, 10, 31, 10, 0],
        b'M' => [17, 27, 21, 21, 17, 17, 17],
        b'W' => [17, 17, 17, 21, 21, 27, 17],
        b'@' => [14, 17, 23, 21, 23, 16, 14],
        b'<' => [2, 4, 8, 16, 8, 4, 2],
        b'>' => [8, 4, 2, 1, 2, 4, 8],
        b'^' => [4, 10, 17, 0, 0, 0, 0],
        b'v' => [0, 0, 17, 17, 10, 10, 4],
        b'[' => [14, 8, 8, 8, 8, 8, 14],
        b']' => [14, 2, 2, 2, 2, 2, 14],
        b'(' => [2, 4, 8, 8, 8, 4, 2],
        b')' => [8, 4, 2, 2, 2, 4, 8],
        _ => [0; 7],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn masks_and_coverage_are_deterministic() {
        let a = Atlas::builtin(6, 10, false);
        let b = Atlas::builtin(6, 10, false);
        assert_eq!(a.glyphs[0].features.mean, 0.);
        for (g, h) in a.glyphs.iter().zip(b.glyphs) {
            assert_eq!(g.mask.len(), 60);
            assert_eq!(g.mask, h.mask);
            assert!((g.features.mean - g.mask.iter().sum::<f32>() / 60.).abs() < 1e-6);
        }
        assert!(a.max_density > 0.2 && a.max_density < 0.6);
    }
    #[test]
    fn flat_patch_has_no_shape() {
        let f = Features::extract(&[0.3; 60], 6, 10);
        assert!(f.variance < 1e-10);
        assert_eq!(f.energy, 0.);
    }
    #[test]
    fn orientation_features_distinguish_lines() {
        let a = Atlas::builtin(6, 10, false);
        let v = &a.glyphs.iter().find(|g| g.byte == b'|').unwrap().features;
        let h = &a.glyphs.iter().find(|g| g.byte == b'-').unwrap().features;
        assert!(v.small[0] > v.small[1]);
        assert!(h.small[1] > h.small[0]);
    }
}
