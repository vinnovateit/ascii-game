//! Reproducible art-direction viewpoints; no changes to the world or camera math.
use crate::engine::camera::Camera;
use glam::Vec3;
pub fn camera(name: &str, fov: f32) -> Option<Camera> {
    let (position, yaw, pitch) = match name {
        "room" | "doorway" => (Vec3::new(0., 1.65, 5.), 0., 0.),
        "crates" => (Vec3::new(-0.4, 1.65, 1.8), -0.62, -0.14),
        "stairs" => (Vec3::new(0., 1.65, -8.), 0., -0.12),
        "landing" => (Vec3::new(0., 2.85, -15.5), std::f32::consts::PI, -0.2),
        _ => return None,
    };
    Some(Camera {
        position,
        yaw,
        pitch,
        fov,
        ..Camera::default()
    })
}
