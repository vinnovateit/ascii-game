use ascii_game::{
    engine::camera::Camera,
    render::{
        ascii::Ascii,
        framebuffer::Framebuffer,
        scene::{Mode, render},
    },
    world::test_scene::test_scene,
};
use glam::Vec3;
fn snapshot(camera: Camera) -> String {
    let world = test_scene();
    let mut fb = Framebuffer::new(120, 40);
    render(&world.meshes, &camera, &mut fb, 0.5, Mode::Ascii);
    assert!(
        fb.depth
            .iter()
            .all(|z| z.is_finite() && (0.0..=1.0).contains(z)),
        "closed room should cover viewport"
    );
    let mut cells = vec![b' '; 4800];
    Ascii::default().convert(&fb, &mut cells, false);
    cells
        .chunks(120)
        .map(|row| format!("{}\n", std::str::from_utf8(row).unwrap()))
        .collect()
}
#[test]
fn room_snapshot() {
    assert_eq!(
        snapshot(Camera::default()),
        include_str!("snapshots/room.txt")
    );
}
#[test]
fn stairs_snapshot() {
    assert_eq!(
        snapshot(Camera {
            position: Vec3::new(0., 1.65, -8.),
            pitch: -0.12,
            ..Camera::default()
        }),
        include_str!("snapshots/stairs.txt")
    );
}
#[test]
fn all_debug_modes_render() {
    let world = test_scene();
    let camera = Camera::default();
    let mut fb = Framebuffer::new(120, 40);
    for mode in [
        Mode::Ascii,
        Mode::Luminance,
        Mode::Depth,
        Mode::Wireframe,
        Mode::Lighting,
    ] {
        let stats = render(&world.meshes, &camera, &mut fb, 0.5, mode);
        assert!(stats.rasterized > 0);
        assert!(
            fb.luminance
                .iter()
                .all(|l| l.is_finite() && (0.0..=1.0).contains(l))
        );
        assert!(fb.luminance.iter().any(|l| *l > 0.));
    }
}
