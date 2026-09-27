use ascii_game::{
    render::{
        ascii::Ascii,
        framebuffer::Framebuffer,
        glyphs::{self, GlyphSet, HYBRID_GLYPHS},
        scene::{Mode, render},
    },
    viewpoints,
    world::test_scene::test_scene,
};
use unicode_width::UnicodeWidthChar;
#[test]
fn frozen_pipeline_and_space_mask_match_for_both_vocabularies() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let world = test_scene();
    for view in ["doorway", "crates", "stairs"] {
        let camera = viewpoints::camera(view, 70f32.to_radians()).unwrap();
        let mut fb = Framebuffer::new(120, 40);
        render(&world.meshes, &camera, &mut fb, 0.5, Mode::Ascii);
        let mut mapper = Ascii::default();
        let mut canonical = vec![b' '; 4800];
        mapper.convert(&fb, &camera, &mut canonical, false);
        let mut ascii = vec![' '; 4800];
        let mut hybrid = ascii.clone();
        GlyphSet::Ascii.resolve_frame(&canonical, 120, &mut ascii);
        GlyphSet::Hybrid.resolve_frame(&canonical, 120, &mut hybrid);
        assert_eq!(
            glyphs::text(&ascii, 120),
            std::fs::read_to_string(
                root.join("visual/pure-ascii/after")
                    .join(view)
                    .join("ascii.txt")
            )
            .unwrap()
        );
        for (a, h) in ascii.iter().zip(&hybrid) {
            assert_eq!(*a == ' ', *h == ' ');
            assert_eq!(h.width(), Some(1));
            assert!(h.is_ascii() || HYBRID_GLYPHS.contains(h));
        }
        assert_ne!(ascii, hybrid);
        let first = hybrid.clone();
        GlyphSet::Hybrid.resolve_frame(&canonical, 120, &mut hybrid);
        assert_eq!(first, hybrid);
        let blocks = hybrid
            .iter()
            .filter(|c| matches!(c, '░' | '▒' | '▓' | '█'))
            .count();
        assert!(
            blocks > 0 && blocks < canonical.len() / 20,
            "shade blocks must stay a small accent: {blocks}"
        );
        for (name, frame) in [("ascii", ascii), ("hybrid", hybrid)] {
            let dir = root.join("visual/glyph-sets").join(name).join(view);
            let saved = std::fs::read_to_string(dir.join("frame.txt")).unwrap();
            assert_eq!(saved, glyphs::text(&frame, 120));
            assert_eq!(saved.lines().count(), 40);
            assert!(saved.lines().all(|row| row.chars().count() == 120));
        }
        for file in [
            "ascii.txt",
            "luminance.pgm",
            "depth.pgm",
            "edges.pgm",
            "density.pgm",
            "style.txt",
            "metrics.txt",
        ] {
            let base = root.join("visual/glyph-sets");
            assert_eq!(
                std::fs::read(base.join("ascii").join(view).join(file)).unwrap(),
                std::fs::read(base.join("hybrid").join(view).join(file)).unwrap(),
                "{view}/{file}"
            );
        }
    }
}
#[test]
fn cli_default_and_explicit_ascii_are_identical() {
    use std::process::Command;
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_main"))
            .args(args)
            .output()
            .unwrap()
    };
    let implicit = run(&["--snapshot"]);
    let explicit = run(&["--snapshot", "--glyph-set", "ascii"]);
    assert!(implicit.status.success() && explicit.status.success());
    assert_eq!(implicit.stdout, explicit.stdout);
    assert!(
        !run(&["--snapshot", "--glyph-set", "braille"])
            .status
            .success()
    );
}
