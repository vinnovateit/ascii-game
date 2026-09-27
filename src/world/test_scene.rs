use crate::render::mesh::Mesh;
use glam::Vec3;
#[derive(Clone, Copy)]
pub struct Solid {
    pub min: Vec3,
    pub max: Vec3,
}
pub struct World {
    pub meshes: Vec<Mesh>,
    pub solids: Vec<Solid>,
}
impl World {
    fn block(&mut self, min: [f32; 3], max: [f32; 3], albedo: f32) {
        let min = Vec3::from_array(min);
        let max = Vec3::from_array(max);
        self.meshes.push(Mesh::cuboid(min, max, albedo));
        self.solids.push(Solid { min, max });
    }
}
pub fn test_scene() -> World {
    let mut w = World {
        meshes: vec![],
        solids: vec![],
    };
    // Room, doorway to northern corridor, and exterior shell.
    w.block([-6., -0.4, -7.], [6., 0., 7.], 0.65);
    w.block([-6.3, 0., -7.3], [-6., 4.5, 7.3], 0.9);
    w.block([6., 0., -7.3], [6.3, 4.5, 7.3], 0.9);
    w.block([-6., 0., 7.], [6., 4.5, 7.3], 0.8);
    w.block([-6., 0., -7.3], [-1.5, 4.5, -7.], 0.95);
    w.block([1.5, 0., -7.3], [6., 4.5, -7.], 0.95);
    w.block([-1.5, 2.9, -7.3], [1.5, 4.5, -7.], 0.85);
    w.block([-6.3, 4.5, -7.3], [6.3, 4.8, 7.3], 0.7);
    // Doorframe relief and supporting pillars provide readable silhouettes.
    w.block([-1.8, 0., -6.95], [-1.5, 3.2, -6.65], 1.);
    w.block([1.5, 0., -6.95], [1.8, 3.2, -6.65], 1.);
    w.block([-1.8, 2.9, -6.95], [1.8, 3.2, -6.65], 1.);
    w.block([-2., -0.4, -17.], [2., 0., -7.], 0.7);
    w.block([-2.3, 0., -17.3], [-2., 4.5, -7.3], 0.85);
    w.block([2., 0., -17.3], [2.3, 4.5, -7.3], 0.85);
    w.block([-2., 0., -17.3], [2., 4.5, -17.], 0.9);
    w.block([-2.3, 4.5, -17.3], [2.3, 4.8, -7.3], 0.7);
    // Six 20cm steps up to a landing: genuine horizontal and vertical triangles.
    for i in 0..6 {
        let z = -10. - i as f32 * 0.5;
        w.block([-2., 0., z - 0.5], [2., (i + 1) as f32 * 0.2, z], 0.85);
    }
    w.block([-2., 0., -17.], [2., 1.2, -13.], 0.75);
    // Crates and shallow framing ribs, all rendered as solid geometry.
    for (x, z, size) in [
        (-3.6, -2.5, 1.5),
        (3.0, -4., 1.8),
        (2.6, 2., 1.2),
        (-3.8, -4.5, 1.0),
    ] {
        w.block([x, 0., z], [x + size, size, z + size], 0.72);
        for y in [0.12, size - 0.2] {
            w.block(
                [x - 0.025, y, z - 0.035],
                [x + size + 0.025, y + 0.12, z + size + 0.035],
                0.95,
            );
        }
    }
    w
}
