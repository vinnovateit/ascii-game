use ascii_game::{
    engine::camera::Camera,
    render::{
        ascii::Ascii,
        framebuffer::Framebuffer,
        scene::{Mode, render},
        terminal::Terminal,
    },
    world::test_scene::test_scene,
};
fn main() -> std::io::Result<()> {
    let world = test_scene();
    let camera = Camera::default();
    let ascii = Ascii::default();
    let snapshot = std::env::args().any(|a| a == "--snapshot");
    if snapshot {
        let mut fb = Framebuffer::new(120, 40);
        render(&world.meshes, &camera, &mut fb, 0.5, Mode::Ascii);
        let mut cells = vec![b' '; 4800];
        ascii.convert(&fb, &mut cells, false);
        for row in cells.chunks(120) {
            println!("{}", std::str::from_utf8(row).unwrap());
        }
        return Ok(());
    }
    let mut terminal = Terminal::enter()?;
    loop {
        let (w, h) = crossterm::terminal::size()?;
        let mut fb = Framebuffer::new(w as usize, h as usize);
        render(&world.meshes, &camera, &mut fb, 0.5, Mode::Ascii);
        let mut cells = vec![b' '; fb.width * fb.height];
        ascii.convert(&fb, &mut cells, false);
        terminal.present(&cells, w as usize)?;
        if crossterm::event::poll(std::time::Duration::from_millis(16))?
            && let crossterm::event::Event::Key(_) = crossterm::event::read()?
        {
            break;
        }
    }
    Ok(())
}
