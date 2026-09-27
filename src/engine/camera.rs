use glam::{Mat4, Vec3};
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0., 1.65, 5.),
            yaw: 0.,
            pitch: 0.,
            fov: 70f32.to_radians(),
            near: 0.08,
            far: 80.,
        }
    }
}
impl Camera {
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        glam::camera::rh::proj::directx::perspective(self.fov, aspect, self.near, self.far)
            * glam::camera::rh::view::look_to_mat4(self.position, self.forward(), Vec3::Y)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn near_and_far_are_zero_and_one() {
        let c = Camera::default();
        let p = glam::camera::rh::proj::directx::perspective(c.fov, 1., c.near, c.far);
        assert!(p.project_point3(Vec3::new(0., 0., -c.near)).z.abs() < 1e-5);
        assert!((p.project_point3(Vec3::new(0., 0., -c.far)).z - 1.).abs() < 1e-5);
    }
}
