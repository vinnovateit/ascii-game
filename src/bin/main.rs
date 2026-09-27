use ascii_game::render::{framebuffer::Framebuffer, rasterizer::triangle, terminal::Terminal};
use glam::{Mat4, Vec3};
fn main() -> std::io::Result<()> {
    let mut terminal = Terminal::enter()?;
    let (w, h) = crossterm::terminal::size()?;
    let mut fb = Framebuffer::new(w as usize, h as usize);
    let projection = Mat4::perspective_rh(
        70f32.to_radians(),
        ascii_game::render::framebuffer::aspect(w as usize, h as usize, 0.5),
        0.1,
        100.,
    );
    let vertices = [
        Vec3::new(-1., -1., -3.),
        Vec3::new(1., -1., -3.),
        Vec3::new(0., 1., -3.),
    ]
    .map(|v| {
        let p = projection.project_point3(v);
        Vec3::new(
            (p.x + 1.) * 0.5 * w as f32,
            (1. - p.y) * 0.5 * h as f32,
            p.z,
        )
    });
    triangle(&mut fb, vertices, 1., false);
    terminal.present(
        &fb.luminance
            .iter()
            .map(|v| if *v > 0. { b'#' } else { b' ' })
            .collect::<Vec<_>>(),
        w as usize,
    )?;
    loop {
        if let crossterm::event::Event::Key(_) = crossterm::event::read()? {
            break;
        }
    }
    Ok(())
}
