use ascii_game::research::atlas::Atlas;
fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("visual/research/atlas")?;
    let atlas = Atlas::builtin(6, 10, false);
    atlas.save(std::path::Path::new("visual/research/atlas/builtin.pgm"))?;
    let stats = atlas
        .glyphs
        .iter()
        .map(|g| format!("{:?} {:.5}\n", g.byte as char, g.features.mean))
        .collect::<String>();
    std::fs::write("visual/research/atlas/coverage.txt", stats)
}
