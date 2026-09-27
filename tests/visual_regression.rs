use ascii_game::{
    render::{
        ascii::Ascii,
        capture::{Metrics, density},
        framebuffer::Framebuffer,
        scene::{Mode, render},
        style::AsciiStyle,
    },
    viewpoints,
    world::test_scene::test_scene,
};
fn ndc(distance: f32) -> f32 {
    let c = viewpoints::camera("doorway", 70f32.to_radians()).unwrap();
    c.far / (c.far - c.near) - c.near * c.far / ((c.far - c.near) * distance)
}
#[test]
fn three_views_are_sparse_repeatable_and_keep_raw_geometry() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let world = test_scene();
    for view in ["doorway", "crates", "stairs"] {
        let camera = viewpoints::camera(view, 70f32.to_radians()).unwrap();
        for (width, height) in [(120, 40), (200, 60), (1, 1)] {
            let mut fb = Framebuffer::new(width, height);
            render(&world.meshes, &camera, &mut fb, 0.5, Mode::Ascii);
            let mut mapper = Ascii::default();
            let mut cells = vec![b' '; width * height];
            mapper.convert(&fb, &camera, &mut cells, false);
            let first = cells.clone();
            mapper.convert(&fb, &camera, &mut cells, false);
            assert_eq!(first, cells);
            assert_eq!(cells.len(), width * height);
            assert!(cells.iter().all(|c| (32..=126).contains(c)));
            if width == 1 {
                continue;
            }
            let metrics = Metrics::measure(&fb, &cells, &camera);
            assert!(
                (0.4..0.76).contains(&metrics.occupancy),
                "{view} {}",
                metrics.occupancy
            );
            assert!(
                (0.15..0.45).contains(&metrics.density),
                "{view} {}",
                metrics.density
            );
            let upper = &cells[..width * (height / 4)];
            assert!(upper.iter().filter(|c| **c == b' ').count() > upper.len() / 3);
            if width == 120 {
                let capture = root.join("visual/after").join(view);
                let text: String = cells
                    .chunks(width)
                    .map(|row| format!("{}\n", std::str::from_utf8(row).unwrap()))
                    .collect();
                assert_eq!(
                    text,
                    std::fs::read_to_string(capture.join("ascii.txt")).unwrap()
                );
                let before = root.join("visual/before").join(view);
                // Captured source buffers must be byte-identical: art direction only.
                for file in ["luminance.pgm", "depth.pgm"] {
                    assert_eq!(
                        std::fs::read(before.join(file)).unwrap(),
                        std::fs::read(capture.join(file)).unwrap()
                    );
                }
                let mut raw = format!("P5\n{width} {height}\n255\n").into_bytes();
                raw.extend(
                    fb.luminance
                        .iter()
                        .map(|l| (l.clamp(0., 1.) * 255.).round() as u8),
                );
                assert_eq!(raw, std::fs::read(capture.join("luminance.pgm")).unwrap());
                let mut depth = format!("P5\n{width} {height}\n255\n").into_bytes();
                depth.extend(fb.depth.iter().map(|z| {
                    let d = ascii_game::render::capture::linear_depth(*z, &camera);
                    ((1. - (d / camera.far).sqrt()).clamp(0., 1.) * 255.).round() as u8
                }));
                assert_eq!(depth, std::fs::read(capture.join("depth.pgm")).unwrap());
                let old = std::fs::read(before.join("ascii.txt")).unwrap();
                let old_occupancy = old.iter().filter(|c| **c != b' ' && **c != b'\n').count()
                    as f32
                    / (width * height) as f32;
                assert!(old_occupancy - metrics.occupancy > 0.2);
            }
        }
    }
}
#[test]
fn identical_lighting_gets_sparser_with_distance() {
    let camera = viewpoints::camera("doorway", 70f32.to_radians()).unwrap();
    let mut fb = Framebuffer::new(80, 20);
    fb.luminance.fill(0.65);
    let mut mapper = Ascii::default();
    let mut cells = vec![b' '; 1600];
    let mut means = vec![];
    for distance in [2., 10., 30.] {
        fb.depth.fill(ndc(distance));
        mapper.convert(&fb, &camera, &mut cells, false);
        means.push(cells.iter().map(|c| density(*c)).sum::<f32>() / 1600.);
    }
    assert!(means[0] > means[1] && means[1] > means[2], "{means:?}");
    mapper.style.depth_falloff = 0.;
    fb.depth.fill(ndc(2.));
    mapper.convert(&fb, &camera, &mut cells, false);
    let near = cells.clone();
    fb.depth.fill(ndc(30.));
    mapper.convert(&fb, &camera, &mut cells, false);
    assert_eq!(near, cells);
}
#[test]
fn empty_pixels_and_dark_edges_remain_spaces() {
    let camera = viewpoints::camera("doorway", 70f32.to_radians()).unwrap();
    let mut fb = Framebuffer::new(12, 8);
    fb.luminance.fill(1.);
    let mut mapper = Ascii::default();
    mapper.style.dither_strength = 2.;
    mapper.style.edge_weight = 1.;
    let mut cells = vec![b'?'; 96];
    mapper.convert(&fb, &camera, &mut cells, false);
    assert!(cells.iter().all(|c| *c == b' '));
    fb.depth.fill(ndc(2.));
    for y in 0..8 {
        for x in 0..6 {
            fb.luminance[y * 12 + x] = 0.;
        }
    }
    mapper.convert(&fb, &camera, &mut cells, false);
    for row in cells.chunks(12) {
        assert!(row[..6].iter().all(|c| *c == b' '));
        assert_eq!(row[6], b'|');
    }
}
#[test]
fn neutral_and_zero_edge_weight_preserve_luminance_mapping() {
    let camera = viewpoints::camera("doorway", 70f32.to_radians()).unwrap();
    let mut fb = Framebuffer::new(80, 20);
    fb.depth.fill(ndc(2.));
    fb.luminance.fill(0.6);
    let mut mapper = Ascii::default();
    mapper.style = AsciiStyle::preset("neutral").unwrap();
    let mut cells = vec![b' '; 1600];
    mapper.convert(&fb, &camera, &mut cells, false);
    assert!(cells.iter().all(|c| *c == cells[0]));
    assert_eq!(mapper.tone(0.6), 0.6);
}
