use ascii_game::{
    render::{
        ascii::Ascii,
        framebuffer::{Framebuffer, aspect},
    },
    research::{
        atlas::Atlas,
        backend::{Backend, Experimental},
    },
    viewpoints,
    world::test_scene::test_scene,
};
#[test]
fn matched_and_enriched_finalists_are_repeatable_and_match_captures() {
    let world = test_scene();
    for kind in [Backend::Matched, Backend::Enriched] {
        for view in ["doorway", "crates", "stairs"] {
            let camera = viewpoints::camera(view, 70f32.to_radians()).unwrap();
            let mut renderer = Experimental::new(kind, (3, 6), 0.5, None).unwrap();
            let mut fb = Framebuffer::new(120, 40);
            let mut ascii = Ascii::default();
            let mut cells = vec![b' '; 4800];
            renderer.render(&world.meshes, &camera, &mut fb, &mut ascii, &mut cells);
            let first = cells.clone();
            renderer.render(&world.meshes, &camera, &mut fb, &mut ascii, &mut cells);
            assert_eq!(cells, first);
            let text = cells
                .chunks(120)
                .map(|row| String::from_utf8(row.to_vec()).unwrap() + "\n")
                .collect::<String>();
            let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("visual/research/final")
                .join(kind.name())
                .join(view)
                .join("frame.txt");
            assert_eq!(text, std::fs::read_to_string(file).unwrap());
            assert!(cells.iter().all(|c| (32..=126).contains(c)));
            renderer.render(&[], &camera, &mut fb, &mut ascii, &mut cells);
            assert!(
                cells.iter().all(|c| *c == b' '),
                "disoccluded background cannot retain a glyph"
            );
        }
    }
}
#[test]
fn supersampling_preserves_physical_viewport() {
    for (w, h) in [(2, 4), (3, 6), (4, 8), (6, 10), (8, 16)] {
        assert!(
            (aspect(120 * w, 40 * h, 0.5 * h as f32 / w as f32) - aspect(120, 40, 0.5)).abs()
                < 1e-6
        );
    }
}
#[test]
fn optional_atlas_loader_works_without_any_font_runtime() {
    let path = std::env::temp_dir().join(format!("ascii-atlas-{}.pgm", std::process::id()));
    let mut data = b"P5\n380 8\n255\n".to_vec();
    let mut pixels = vec![0u8; 380 * 8];
    pixels[(b'x' - 32) as usize * 4] = 255;
    data.extend(pixels);
    std::fs::write(&path, data).unwrap();
    let a = Atlas::load(&path, 4, 8, false).unwrap();
    assert!(Atlas::load(&path, 3, 6, false).is_err());
    std::fs::remove_file(path).unwrap();
    assert_eq!(a.glyphs[0].features.mean, 0.);
    assert!(
        a.glyphs
            .iter()
            .find(|g| g.byte == b'x')
            .unwrap()
            .features
            .mean
            > 0.
    );
}

#[test]
fn cli_keeps_fast_default_and_rejects_ignored_matched_tuning() {
    use std::process::Command;
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_main"))
            .args(args)
            .output()
            .unwrap()
    };
    let default = run(&["--snapshot"]);
    let fast = run(&["--snapshot", "--ascii-backend", "fast"]);
    assert!(default.status.success() && fast.status.success());
    assert_eq!(default.stdout, fast.stdout);
    for args in [
        vec!["--snapshot", "--ascii-backend", "matched", "--gamma", "2"],
        vec![
            "--snapshot",
            "--ascii-backend",
            "matched",
            "--match-samples",
            "0x0",
        ],
        vec!["--snapshot", "--ascii-backend", "unknown"],
    ] {
        assert!(!run(&args).status.success());
    }
}
