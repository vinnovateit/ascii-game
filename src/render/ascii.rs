use super::framebuffer::Framebuffer;
pub struct Ascii {
    pub contrast: f32,
    pub gamma: f32,
    pub levels: usize,
    pub dither: f32,
    pub bands: Vec<Vec<u8>>,
}
impl Default for Ascii {
    fn default() -> Self {
        Self {
            contrast: 1.35,
            gamma: 0.85,
            levels: 7,
            dither: 0.7,
            bands: vec![
                b" ".to_vec(),
                b".,'`".to_vec(),
                b":;!i".to_vec(),
                b"+=*x".to_vec(),
                b"X$&".to_vec(),
                b"#%MW".to_vec(),
                b"@N".to_vec(),
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
    pub fn glyph(&self, luminance: f32, x: usize, y: usize) -> u8 {
        const BAYER: [[f32; 4]; 4] = [
            [0., 8., 2., 10.],
            [12., 4., 14., 6.],
            [3., 11., 1., 9.],
            [15., 7., 13., 5.],
        ];
        let l = ((luminance - 0.5) * self.contrast + 0.5)
            .clamp(0., 1.)
            .powf(1. / self.gamma.max(0.01));
        let levels = self.levels.max(2) as f32 - 1.;
        let l = ((l * levels + (BAYER[y % 4][x % 4] / 16. - 0.46875) * self.dither).round()
            / levels)
            .clamp(0., 1.);
        if self.bands.is_empty() {
            return b' ';
        }
        let band = &self.bands[(l * (self.bands.len() - 1) as f32).round() as usize];
        if band.is_empty() {
            b' '
        } else {
            band[hash(x, y) as usize % band.len()]
        }
    }
    pub fn convert(&self, fb: &Framebuffer, cells: &mut [u8], plain: bool) {
        for (i, (l, cell)) in fb.luminance.iter().zip(cells).enumerate() {
            *cell = if plain {
                b" .:-=+*#%@"[(l.clamp(0., 1.) * 9.).round() as usize]
            } else {
                self.glyph(*l, i % fb.width, i / fb.width)
            };
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_variants_and_bands() {
        let a = Ascii::default();
        assert_eq!(a.glyph(0., 2, 2), b' ');
        let first: Vec<_> = (0..64).map(|x| a.glyph(0.6, x, 7)).collect();
        assert_eq!(
            first,
            (0..64).map(|x| a.glyph(0.6, x, 7)).collect::<Vec<_>>()
        );
        assert!(first.windows(2).any(|p| p[0] != p[1]));
        assert_ne!(hash(3, 5), hash(5, 3));
    }
    #[test]
    fn ramp_rejects_terminal_controls_and_unicode() {
        let mut a = Ascii::default();
        assert!(a.set_ramp("\x1b").is_err());
        assert!(a.set_ramp("█").is_err());
        assert!(a.set_ramp(" .#").is_ok());
    }
}
