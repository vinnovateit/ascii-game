use ascii_game::{
    engine::camera::Camera,
    render::{
        framebuffer::Framebuffer,
        mesh::Mesh,
        scene::{Mode, render},
        terminal::Terminal,
    },
};
use glam::{Mat4, Vec3};
fn main() -> std::io::Result<()> {
    let mut terminal = Terminal::enter()?;
    let start = std::time::Instant::now();
    loop {
        let (w, h) = crossterm::terminal::size()?;
        let mut fb = Framebuffer::new(w as usize, h as usize);
        let mut cube = Mesh::cuboid(Vec3::splat(-1.), Vec3::ONE, 1.);
        cube.model = Mat4::from_rotation_y(start.elapsed().as_secs_f32());
        render(
            &[cube],
            &Camera {
                position: Vec3::new(0., 0., 5.),
                ..Camera::default()
            },
            &mut fb,
            0.5,
            Mode::Ascii,
        );
        let ramp = b" .:-=+*#%@";
        terminal.present(
            &fb.luminance
                .iter()
                .map(|v| ramp[(v.clamp(0., 1.) * 9.) as usize])
                .collect::<Vec<_>>(),
            w as usize,
        )?;
        if crossterm::event::poll(std::time::Duration::from_millis(16))?
            && let crossterm::event::Event::Key(_) = crossterm::event::read()?
        {
            break;
        }
    }
    Ok(())
}
