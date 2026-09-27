//! Experimental grayscale source enrichment after the unchanged rasterizer.
use crate::{
    engine::camera::Camera,
    render::{
        capture::linear_depth,
        framebuffer::{Framebuffer, aspect},
    },
};
use glam::{Vec2, Vec3, Vec4};
#[derive(Clone, Copy, Debug, Default)]
pub struct SourceSettings {
    pub noise: f32,
    pub contact: f32,
    pub lighting: bool,
    pub context: f32,
    pub edge_boost: f32,
}
#[derive(Default)]
pub struct SourcePass {
    positions: Vec<Vec3>,
    depths: Vec<f32>,
    original: Vec<f32>,
}
impl SourcePass {
    pub fn apply(
        &mut self,
        fb: &mut Framebuffer,
        camera: &Camera,
        pixel_ratio: f32,
        s: SourceSettings,
    ) {
        if s.noise == 0. && s.contact == 0. && !s.lighting && s.context == 0. && s.edge_boost == 0.
        {
            return;
        }
        let w = fb.width;
        let h = fb.height;
        self.positions.resize(w * h, Vec3::ZERO);
        self.depths.resize(w * h, 0.);
        let inverse = camera.matrix(aspect(w, h, pixel_ratio)).inverse();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                self.depths[i] = linear_depth(fb.depth[i], camera);
                if fb.depth[i].is_finite() {
                    let p = inverse
                        * Vec4::new(
                            2. * (x as f32 + 0.5) / w as f32 - 1.,
                            1. - 2. * (y as f32 + 0.5) / h as f32,
                            fb.depth[i],
                            1.,
                        );
                    self.positions[i] = p.truncate() / p.w;
                }
            }
        }
        let sun = Vec3::new(-0.4, 0.85, 0.3).normalize();
        let fill = Vec3::new(0.6, 0.4, -0.6).normalize();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if !fb.depth[i].is_finite() {
                    continue;
                }
                let p = self.positions[i];
                // Pick the closer neighbor on each axis to avoid normals across depth edges.
                let a = self.positions[y * w + x.saturating_sub(1)];
                let b = self.positions[y * w + (x + 1).min(w - 1)];
                let c = self.positions[y.saturating_sub(1) * w + x];
                let d = self.positions[(y + 1).min(h - 1) * w + x];
                let dx = if p.distance_squared(a) < p.distance_squared(b) {
                    p - a
                } else {
                    b - p
                };
                let dy = if p.distance_squared(c) < p.distance_squared(d) {
                    p - c
                } else {
                    d - p
                };
                let normal = dy.cross(dx).normalize_or_zero();
                let mut l = fb.luminance[i];
                if s.lighting {
                    let albedo = l / (0.32 + 0.68 * normal.dot(sun).max(0.));
                    l = albedo
                        * (0.12
                            + 0.76 * normal.dot(sun).max(0.)
                            + 0.10 * normal.dot(fill).max(0.)
                            + 0.04 * normal.y.max(0.));
                }
                if s.noise > 0. {
                    let n = normal.abs();
                    let uv = if n.y >= n.x && n.y >= n.z {
                        Vec2::new(p.x, p.z)
                    } else if n.x >= n.z {
                        Vec2::new(p.z, p.y)
                    } else {
                        Vec2::new(p.x, p.y)
                    };
                    let footprint = dx.length_squared().max(dy.length_squared()).sqrt();
                    let coarse = (1. - (footprint * 3. - 0.3) / 0.7).clamp(0., 1.);
                    let fine = (1. - (footprint * 23. - 0.3) / 0.7).clamp(0., 1.);
                    let detail = if coarse > 0. {
                        0.55 * coarse * (value_noise(uv * 3.) - 0.5)
                    } else {
                        0.
                    } + if fine > 0. {
                        0.45 * fine * (value_noise(uv * 23.) - 0.5)
                    } else {
                        0.
                    };
                    l += s.noise * detail;
                }
                if s.contact > 0. {
                    let z = self.depths[i];
                    let radius = (z * camera.fov.mul_add(0.5, 0.).tan() / h as f32 * 10.).max(0.04);
                    let mut occlusion = 0.;
                    for (ox, oy) in [(-3, 0), (3, 0), (0, -3), (0, 3), (-2, -2), (2, 2)] {
                        let nx = (x as isize + ox).clamp(0, w as isize - 1) as usize;
                        let ny = (y as isize + oy).clamp(0, h as isize - 1) as usize;
                        let j = ny * w + nx;
                        let separation = self.positions[j] - p;
                        let delta = normal.dot(separation);
                        // Plane-relative separation avoids falsely shadowing sloped
                        // planes simply because their view-space depth changes.
                        if fb.depth[j].is_finite()
                            && delta > radius * 0.08
                            && separation.length_squared() < (radius * 5.).powi(2)
                        {
                            occlusion += (1. - separation.length() / (radius * 5.)).max(0.);
                        }
                    }
                    l *= 1. - s.contact * occlusion / 6.;
                }
                fb.luminance[i] = l.clamp(0., 1.);
            }
        }
        if s.context > 0. || s.edge_boost > 0. {
            self.original.clone_from(&fb.luminance);
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    if !fb.depth[i].is_finite() {
                        continue;
                    }
                    let mut mean = 0.;
                    let mut count: f32 = 0.;
                    for (ox, oy) in [(-6, 0), (6, 0), (0, -10), (0, 10)] {
                        let nx = (x as isize + ox).clamp(0, w as isize - 1) as usize;
                        let ny = (y as isize + oy).clamp(0, h as isize - 1) as usize;
                        if fb.depth[ny * w + nx].is_finite() {
                            mean += self.original[ny * w + nx];
                            count += 1.;
                        }
                    }
                    let gx = self.original[y * w + (x + 1).min(w - 1)]
                        - self.original[y * w + x.saturating_sub(1)];
                    let gy = self.original[(y + 1).min(h - 1) * w + x]
                        - self.original[y.saturating_sub(1) * w + x];
                    fb.luminance[i] = (self.original[i]
                        + s.context * (self.original[i] - mean / count.max(1.))
                        + s.edge_boost * gx.hypot(gy))
                    .clamp(0., 1.);
                }
            }
        }
    }
}
fn lattice(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6b343) ^ (y as u32).wrapping_mul(0xd8163841);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    (h & 65535) as f32 / 65535.
}
pub fn value_noise(p: Vec2) -> f32 {
    let cell = p.floor();
    let f = p - cell;
    let t = f * f * (Vec2::splat(3.) - 2. * f);
    let x = cell.x as i32;
    let y = cell.y as i32;
    let a = lattice(x, y);
    let b = lattice(x + 1, y);
    let c = lattice(x, y + 1);
    let d = lattice(x + 1, y + 1);
    (a + (b - a) * t.x) * (1. - t.y) + (c + (d - c) * t.x) * t.y
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noise_is_world_attached_bounded_and_continuous() {
        let p = Vec2::new(-2.3, 8.7);
        assert_eq!(value_noise(p), value_noise(p));
        assert!((0.0..=1.0).contains(&value_noise(p)));
        assert!((value_noise(p) - value_noise(p + Vec2::splat(1e-4))).abs() < 0.002);
    }
    #[test]
    fn unprojection_roundtrip() {
        let c = Camera::default();
        let m = c.matrix(1.5);
        let p = Vec3::new(0.4, 1., -2.);
        let v = m.project_point3(p);
        assert!(m.inverse().project_point3(v).distance(p) < 1e-4);
    }
    #[test]
    fn contact_pass_does_not_shadow_a_sloping_depth_plane() {
        use crate::render::{
            mesh::Mesh,
            scene::{Mode, render},
        };
        let camera = Camera {
            pitch: -0.5,
            ..Camera::default()
        };
        let floor = Mesh::cuboid(Vec3::new(-100., -1., -100.), Vec3::new(100., 0., 100.), 0.7);
        let mut fb = Framebuffer::new(60, 40);
        render(&[floor], &camera, &mut fb, 1., Mode::Ascii);
        let before = fb.luminance.clone();
        SourcePass::default().apply(
            &mut fb,
            &camera,
            1.,
            SourceSettings {
                contact: 1.,
                ..SourceSettings::default()
            },
        );
        for y in 4..36 {
            for x in 4..56 {
                let i = y * 60 + x;
                if fb.depth[i].is_finite() {
                    assert!((fb.luminance[i] - before[i]).abs() < 0.001);
                }
            }
        }
    }
}
