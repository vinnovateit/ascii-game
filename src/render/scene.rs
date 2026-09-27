use super::{
    clipping::clip,
    framebuffer::{Framebuffer, aspect},
    mesh::Mesh,
    rasterizer::triangle,
};
use crate::engine::camera::Camera;
use glam::Vec3;
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Mode {
    #[default]
    Ascii,
    Luminance,
    Depth,
    Wireframe,
    Lighting,
}
#[derive(Default)]
pub struct Stats {
    pub triangles: usize,
    pub rasterized: usize,
}
pub fn render(
    meshes: &[Mesh],
    camera: &Camera,
    fb: &mut Framebuffer,
    cell_ratio: f32,
    mode: Mode,
) -> Stats {
    fb.clear();
    let mut stats = Stats::default();
    let vp = camera.matrix(aspect(fb.width, fb.height, cell_ratio));
    let sun = Vec3::new(-0.4, 0.85, 0.3).normalize();
    for mesh in meshes {
        for indices in &mesh.triangles {
            stats.triangles += 1;
            let world = indices.map(|i| mesh.model.transform_point3(mesh.vertices[i]));
            let normal = (world[1] - world[0])
                .cross(world[2] - world[0])
                .normalize_or_zero();
            if normal.dot(camera.position - world[0]) <= 0. {
                continue;
            }
            let diffuse = 0.22 + 0.78 * normal.dot(sun).max(0.);
            let light = if mode == Mode::Wireframe {
                1.
            } else if mode == Mode::Lighting {
                diffuse
            } else {
                diffuse * mesh.albedo
            };
            let (vertices, n) = clip(world.map(|v| vp * v.extend(1.)));
            for i in 1..n.saturating_sub(1) {
                let projected = [vertices[0], vertices[i], vertices[i + 1]].map(|v| {
                    let p = v.truncate() / v.w;
                    Vec3::new(
                        (p.x + 1.) * 0.5 * fb.width as f32,
                        (1. - p.y) * 0.5 * fb.height as f32,
                        p.z,
                    )
                });
                triangle(fb, projected, light, mode == Mode::Wireframe);
                stats.rasterized += 1;
            }
        }
    }
    if mode == Mode::Depth {
        for (l, z) in fb.luminance.iter_mut().zip(&fb.depth) {
            *l = if z.is_finite() {
                let distance =
                    camera.near * camera.far / (camera.far - z * (camera.far - camera.near));
                1. - (distance / camera.far).sqrt()
            } else {
                0.
            };
        }
    }
    stats
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotating_cube_and_multiple_meshes() {
        let mut cube = Mesh::cuboid(Vec3::splat(-1.), Vec3::ONE, 0.9);
        let camera = Camera {
            position: Vec3::new(0., 0., 5.),
            ..Camera::default()
        };
        let mut fb = Framebuffer::new(80, 40);
        render(
            std::slice::from_ref(&cube),
            &camera,
            &mut fb,
            0.5,
            Mode::Ascii,
        );
        let first = fb.luminance.clone();
        cube.model = glam::Mat4::from_rotation_y(0.6);
        let other = Mesh::cuboid(Vec3::new(1.5, -1., -2.), Vec3::new(3., 1., 0.), 0.7);
        let stats = render(&[cube, other], &camera, &mut fb, 0.5, Mode::Ascii);
        assert_eq!(stats.triangles, 24);
        assert!(stats.rasterized > 2);
        assert_ne!(first, fb.luminance);
        assert!(fb.depth.iter().any(|z| z.is_finite()));
    }
}
