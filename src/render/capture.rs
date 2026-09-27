use super::framebuffer::Framebuffer;
use crate::engine::camera::Camera;
use std::{fs, io, path::Path};
/// Font-independent density proxy, fixed across both old and new palettes.
/// This is an ordinal ink estimate, not measured font pixel coverage.
pub fn density(c: u8) -> f32 {
    match c {
        b' ' => 0.,
        b'.' | b'\'' | b'`' => 0.12,
        b',' | b':' => 0.22,
        b';' | b'!' | b'i' | b'|' | b'/' | b'\\' | b'-' | b'_' | b'~' => 0.32,
        b'+' | b'=' => 0.45,
        b'*' | b'x' => 0.55,
        b'X' | b'$' | b'&' => 0.65,
        b'#' | b'%' => 0.8,
        b'M' | b'W' | b'@' | b'N' => 1.,
        _ => 0.5,
    }
}
pub fn linear_depth(z: f32, camera: &Camera) -> f32 {
    if !z.is_finite() {
        return f32::INFINITY;
    }
    camera.near * camera.far / (camera.far - z * (camera.far - camera.near))
}
pub struct Metrics {
    pub non_space: usize,
    pub occupancy: f32,
    pub density: f32,
    pub luminance: f32,
    pub depth: f32,
}
impl Metrics {
    pub fn measure(fb: &Framebuffer, cells: &[u8], camera: &Camera) -> Self {
        let n = cells.len().max(1) as f32;
        let non_space = cells.iter().filter(|c| **c != b' ').count();
        let (depth, count) = fb
            .depth
            .iter()
            .map(|z| linear_depth(*z, camera))
            .filter(|z| z.is_finite())
            .fold((0., 0usize), |(sum, n), z| (sum + z, n + 1));
        Self {
            non_space,
            occupancy: non_space as f32 / n,
            density: cells.iter().map(|c| density(*c)).sum::<f32>() / n,
            luminance: fb.luminance.iter().sum::<f32>() / n,
            depth: depth / count.max(1) as f32,
        }
    }
    pub fn text(&self) -> String {
        format!(
            "non_space={}\nnon_space_percent={:.3}\nmean_glyph_density={:.5}\nmean_raw_luminance={:.5}\nmean_depth_world_units={:.5}\n",
            self.non_space,
            self.occupancy * 100.,
            self.density,
            self.luminance,
            self.depth
        )
    }
}
pub fn pgm(
    path: &Path,
    width: usize,
    height: usize,
    values: impl Iterator<Item = f32>,
) -> io::Result<()> {
    let mut data = format!("P5\n{width} {height}\n255\n").into_bytes();
    data.extend(values.map(|v| (v.clamp(0., 1.) * 255.).round() as u8));
    fs::write(path, data)
}
pub fn save(path: &Path, fb: &Framebuffer, cells: &[u8], camera: &Camera) -> io::Result<()> {
    fs::create_dir_all(path)?;
    let mut text = Vec::with_capacity(cells.len() + fb.height);
    for row in cells.chunks(fb.width) {
        text.extend_from_slice(row);
        text.push(b'\n');
    }
    fs::write(path.join("ascii.txt"), text)?;
    pgm(
        &path.join("luminance.pgm"),
        fb.width,
        fb.height,
        fb.luminance.iter().copied(),
    )?;
    pgm(
        &path.join("depth.pgm"),
        fb.width,
        fb.height,
        fb.depth.iter().map(|z| {
            let d = linear_depth(*z, camera);
            if d.is_finite() {
                1. - (d / camera.far).sqrt()
            } else {
                0.
            }
        }),
    )?;
    pgm(
        &path.join("density.pgm"),
        fb.width,
        fb.height,
        cells.iter().map(|c| density(*c)),
    )?;
    fs::write(
        path.join("metrics.txt"),
        format!(
            "width={}\nheight={}\nposition={:?}\nyaw={}\npitch={}\nfov={}\nnear={}\nfar={}\n{}",
            fb.width,
            fb.height,
            camera.position,
            camera.yaw,
            camera.pitch,
            camera.fov,
            camera.near,
            camera.far,
            Metrics::measure(fb, cells, camera).text()
        ),
    )
}

/// Append the resolved vocabulary without replacing canonical ASCII/source captures.
pub fn save_glyphs(
    path: &Path,
    width: usize,
    height: usize,
    cells: &[char],
    set: super::glyphs::GlyphSet,
) -> io::Result<()> {
    let frame = super::glyphs::text(cells, width);
    fs::write(path.join("frame.txt"), &frame)?;
    pgm(
        &path.join("glyph-density.pgm"),
        width,
        height,
        cells.iter().map(|c| glyph_density(*c)),
    )?;
    let non_space = cells.iter().filter(|c| **c != ' ').count();
    let unicode = cells.iter().filter(|c| !c.is_ascii()).count();
    let blocks = cells
        .iter()
        .filter(|c| matches!(c, '░' | '▒' | '▓' | '█'))
        .count();
    fs::write(
        path.join("glyphs.txt"),
        format!(
            "glyph_set={}\nwidth={}\nheight={}\nnon_space={}\nnon_space_percent={:.3}\nunicode_cells={}\nblock_cells={}\nutf8_bytes={}\nmean_glyph_density={:.5}\n",
            set.name(),
            width,
            height,
            non_space,
            100. * non_space as f32 / cells.len() as f32,
            unicode,
            blocks,
            frame.len(),
            cells.iter().map(|c| glyph_density(*c)).sum::<f32>() / cells.len() as f32
        ),
    )
}
pub fn glyph_density(c: char) -> f32 {
    match c {
        '░' => 0.25,
        '▒' => 0.5,
        '▓' => 0.75,
        '█' => 1.,
        '─' | '│' | '╱' | '╲' => 0.32,
        _ => density(c as u8),
    }
}
