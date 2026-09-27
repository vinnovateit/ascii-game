use super::framebuffer::Framebuffer;
use glam::{Vec2, Vec3};
pub fn edge(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}
/// Rasterize viewport vertices with NDC depth (affine in screen space).
pub fn triangle(fb: &mut Framebuffer, v: [Vec3; 3], light: f32, wire: bool) {
    let p = v.map(|v| v.truncate());
    let area = edge(p[0], p[1], p[2]);
    if !area.is_finite() || area.abs() < 1e-6 {
        return;
    }
    let min = p[0].min(p[1]).min(p[2]).floor().max(Vec2::ZERO);
    let max = p[0]
        .max(p[1])
        .max(p[2])
        .ceil()
        .min(Vec2::new(fb.width as f32, fb.height as f32));
    for y in min.y as usize..max.y as usize {
        for x in min.x as usize..max.x as usize {
            let q = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let weights = [
                edge(p[1], p[2], q) / area,
                edge(p[2], p[0], q) / area,
                edge(p[0], p[1], q) / area,
            ];
            if weights.iter().any(|w| *w < -1e-6) {
                continue;
            }
            let depth = weights[0] * v[0].z + weights[1] * v[1].z + weights[2] * v[2].z;
            let i = y * fb.width + x;
            if !(0.0..=1.0).contains(&depth) || depth >= fb.depth[i] {
                continue;
            }
            fb.depth[i] = depth;
            let border = (0..3).any(|j| {
                weights[j] * area.abs() / p[(j + 1) % 3].distance(p[(j + 2) % 3]).max(1e-6) < 0.65
            });
            fb.luminance[i] = if wire && !border { 0. } else { light };
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn depth_is_order_independent() {
        let mut fb = Framebuffer::new(8, 8);
        let v = [
            Vec3::new(0., 0., 0.8),
            Vec3::new(8., 0., 0.8),
            Vec3::new(0., 8., 0.8),
        ];
        triangle(&mut fb, v, 0.3, false);
        triangle(&mut fb, v.map(|v| Vec3::new(v.x, v.y, 0.2)), 0.9, false);
        triangle(&mut fb, v, 0.3, false);
        assert_eq!(fb.luminance[9], 0.9);
        assert!((fb.depth[9] - 0.2).abs() < 1e-6);
    }
    #[test]
    fn barycentric_orientation() {
        assert_eq!(edge(Vec2::ZERO, Vec2::X, Vec2::Y), 1.);
    }
}
