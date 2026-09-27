pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub luminance: Vec<f32>,
    pub depth: Vec<f32>,
}
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            luminance: vec![0.; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }
    pub fn clear(&mut self) {
        self.luminance.fill(0.);
        self.depth.fill(f32::INFINITY);
    }
}
/// Physical viewport aspect, given terminal cell width / height.
pub fn aspect(width: usize, height: usize, cell_ratio: f32) -> f32 {
    width.max(1) as f32 * cell_ratio / height.max(1) as f32
}
#[cfg(test)]
mod tests {
    #[test]
    fn physical_aspect() {
        assert_eq!(super::aspect(120, 30, 0.5), 2.);
    }
}
