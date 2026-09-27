use crate::{
    engine::{
        collision::move_player,
        input::Input,
        time::{FixedClock, STEP},
    },
    render::{
        ascii::Ascii,
        framebuffer::Framebuffer,
        scene::{Mode, render},
        terminal::Terminal,
    },
    world::test_scene::test_scene,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use glam::Vec3;
use std::{
    io::{self, IsTerminal},
    time::{Duration, Instant},
};
pub struct Options {
    pub width: usize,
    pub height: usize,
    pub cell_ratio: f32,
    pub fov: f32,
    pub ascii: Ascii,
    pub snapshot: bool,
    pub capture: Option<std::path::PathBuf>,
    pub benchmark: usize,
    pub telemetry: bool,
    pub mode: Mode,
    pub view: String,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            width: 120,
            height: 40,
            cell_ratio: 0.5,
            fov: 70.,
            ascii: Ascii::default(),
            snapshot: false,
            capture: None,
            benchmark: 0,
            telemetry: false,
            mode: Mode::Ascii,
            view: "room".into(),
        }
    }
}
impl Options {
    pub fn parse() -> Result<Option<Self>, String> {
        let mut o = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--help" {
                println!(
                    "ASCII 3D renderer\nWASD move; arrows look; mouse drag/move look; Q/Esc exit; 1-5 debug; T telemetry\n--snapshot  --benchmark FRAMES  --width N  --height N  --view room|stairs|landing\n--fov DEGREES  --cell-aspect RATIO  --contrast N  --gamma N  --levels N\n--dither N  --ramp ' .:#@'  --telemetry  --mode 1..5"
                );
                return Ok(None);
            }
            match arg.as_str() {
                "--snapshot" => o.snapshot = true,
                "--telemetry" => o.telemetry = true,
                _ => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("missing value for {arg}"))?;
                    match arg.as_str() {
                        "--width" => o.width = value.parse().map_err(|_| "invalid width")?,
                        "--height" => o.height = value.parse().map_err(|_| "invalid height")?,
                        "--benchmark" => {
                            o.benchmark = value.parse().map_err(|_| "invalid frame count")?
                        }
                        "--fov" => o.fov = value.parse().map_err(|_| "invalid fov")?,
                        "--cell-aspect" => {
                            o.cell_ratio = value.parse().map_err(|_| "invalid cell aspect")?
                        }
                        "--contrast" => {
                            o.ascii.style.contrast =
                                value.parse().map_err(|_| "invalid contrast")?
                        }
                        "--gamma" => {
                            o.ascii.style.gamma = value.parse().map_err(|_| "invalid gamma")?
                        }
                        "--levels" => {
                            o.ascii.style.posterization_levels =
                                value.parse().map_err(|_| "invalid levels")?
                        }
                        "--dither" => {
                            o.ascii.style.dither_strength =
                                value.parse().map_err(|_| "invalid dither")?
                        }
                        "--ramp" => o.ascii.set_ramp(&value)?,
                        "--view" | "--camera-preset" => o.view = value,
                        "--capture-frame" => o.capture = Some(value.into()),
                        "--mode" => {
                            o.mode = mode(value.parse().map_err(|_| "invalid mode")?)
                                .ok_or("mode must be 1..5")?
                        }
                        _ => return Err(format!("unknown option {arg}")),
                    }
                }
            }
        }
        if !(1..=500).contains(&o.width) || !(1..=240).contains(&o.height) {
            return Err("dimensions must be 1..500 by 1..240".into());
        }
        for (name, value, low, high) in [
            ("fov", o.fov, 30., 120.),
            ("cell aspect", o.cell_ratio, 0.2, 2.),
            ("gamma", o.ascii.style.gamma, 0.1, 4.),
            ("contrast", o.ascii.style.contrast, 0.1, 4.),
            ("dither", o.ascii.style.dither_strength, 0., 2.),
        ] {
            if !value.is_finite() || !(low..=high).contains(&value) {
                return Err(format!("{name} must be {low}..{high}"));
            }
        }
        if !(4..=12).contains(&o.ascii.style.posterization_levels) {
            return Err("levels must be 4..12".into());
        }
        if crate::viewpoints::camera(&o.view, o.fov.to_radians()).is_none() {
            return Err("view must be doorway, crates, stairs, room or landing".into());
        }
        Ok(Some(o))
    }
}
fn mode(n: u8) -> Option<Mode> {
    Some(match n {
        1 => Mode::Ascii,
        2 => Mode::Luminance,
        3 => Mode::Depth,
        4 => Mode::Wireframe,
        5 => Mode::Lighting,
        _ => return None,
    })
}
pub fn run(mut options: Options) -> io::Result<()> {
    let world = test_scene();
    let mut camera = crate::viewpoints::camera(&options.view, options.fov.to_radians()).unwrap();
    if options.snapshot || options.capture.is_some() || options.benchmark > 0 {
        let mut fb = Framebuffer::new(options.width, options.height);
        let mut cells = vec![b' '; fb.width * fb.height];
        let start = Instant::now();
        let mut triangles = 0;
        let mut checksum = 0u64;
        for frame in 0..options.benchmark.max(1) {
            if options.benchmark > 0 {
                camera.yaw = frame as f32 * 0.005;
            }
            triangles = render(
                &world.meshes,
                &camera,
                &mut fb,
                options.cell_ratio,
                options.mode,
            )
            .rasterized;
            options
                .ascii
                .convert(&fb, &mut cells, options.mode != Mode::Ascii);
            checksum = std::hint::black_box(
                cells
                    .iter()
                    .fold(0u64, |h, c| h.wrapping_mul(31).wrapping_add(*c as u64)),
            );
        }
        if options.benchmark > 0 {
            eprintln!(
                "{} frames, {}x{}, {:.3} ms/frame, {:.1} FPS CPU-only, {} rasterized triangles (last frame), checksum {:016x}",
                options.benchmark,
                fb.width,
                fb.height,
                start.elapsed().as_secs_f64() * 1000. / options.benchmark as f64,
                options.benchmark as f64 / start.elapsed().as_secs_f64(),
                triangles,
                checksum
            );
        }
        if let Some(path) = &options.capture {
            crate::render::capture::save(path, &fb, &cells, &camera)?;
        }
        if options.snapshot {
            for row in cells.chunks(fb.width) {
                println!("{}", std::str::from_utf8(row).unwrap());
            }
        }
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "interactive mode requires a terminal; use --snapshot or --benchmark",
        ));
    }
    let mut terminal = Terminal::enter()?;
    let mut input = Input::default();
    input.releases = terminal.key_releases;
    let mut clock = FixedClock::default();
    let mut last = Instant::now();
    let mut velocity = 0.;
    let mut mouse = None;
    let mut fb = Framebuffer::new(1, 1);
    let mut cells = vec![b' '; 1];
    let mut frame_ms = 16.67;
    'running: loop {
        let start = Instant::now();
        let elapsed = start.duration_since(last).as_secs_f32();
        last = start;
        // Bound the event batch so mouse floods cannot starve rendering.
        for _ in 0..256 {
            if !event::poll(Duration::ZERO)? {
                break;
            }
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Release {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            break 'running;
                        }
                        if let KeyCode::Char(c) = key.code {
                            if let Some(m) = c.to_digit(10).and_then(|n| mode(n as u8)) {
                                options.mode = m;
                            }
                            if c == 't' && key.kind == KeyEventKind::Press {
                                options.telemetry = !options.telemetry;
                            }
                        }
                    }
                    input.key(key, start);
                }
                Event::Mouse(e) => {
                    if matches!(e.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) {
                        if let Some((x, y)) = mouse {
                            camera.yaw += (e.column as f32 - x as f32) * 0.012;
                            camera.pitch -= (e.row as f32 - y as f32) * 0.024;
                        }
                        mouse = Some((e.column, e.row));
                    }
                }
                Event::FocusLost => {
                    input.clear();
                    mouse = None;
                }
                Event::Resize(_, _) => {
                    mouse = None;
                }
                _ => {}
            }
        }
        for _ in 0..clock.advance(elapsed) {
            let axis = |a, b| f32::from(input.down(a, start)) - f32::from(input.down(b, start));
            camera.yaw += axis(KeyCode::Right, KeyCode::Left) * 1.7 * STEP;
            camera.pitch += axis(KeyCode::Up, KeyCode::Down) * 1.3 * STEP;
            camera.pitch = camera.pitch.clamp(-1.45, 1.45);
            let forward = Vec3::new(camera.yaw.sin(), 0., -camera.yaw.cos());
            let right = Vec3::new(camera.yaw.cos(), 0., camera.yaw.sin());
            let movement = forward * axis(KeyCode::Char('w'), KeyCode::Char('s'))
                + right * axis(KeyCode::Char('d'), KeyCode::Char('a'));
            move_player(
                &mut camera.position,
                &mut velocity,
                movement.normalize_or_zero() * 3. * STEP,
                STEP,
                &world.solids,
            );
        }
        camera.pitch = camera.pitch.clamp(-1.45, 1.45);
        let (w, h) = crossterm::terminal::size()?;
        let width = (w as usize).clamp(1, 500);
        let height = (h as usize).clamp(1, 240);
        if fb.width != width || fb.height != height {
            fb = Framebuffer::new(width, height);
            cells.resize(width * height, b' ');
        }
        let stats = render(
            &world.meshes,
            &camera,
            &mut fb,
            options.cell_ratio,
            options.mode,
        );
        options
            .ascii
            .convert(&fb, &mut cells, options.mode != Mode::Ascii);
        if options.telemetry {
            let line = format!(
                "{:.0}fps {:.1}ms tri {}/{} fb {}x{} tty {}x{} xyz {:.1},{:.1},{:.1}",
                1000. / frame_ms,
                frame_ms,
                stats.rasterized,
                stats.triangles,
                width,
                height,
                w,
                h,
                camera.position.x,
                camera.position.y,
                camera.position.z
            );
            let row = &mut cells[(height - 1) * width..];
            row.fill(b' ');
            let n = line.len().min(width);
            row[..n].copy_from_slice(&line.as_bytes()[..n]);
        }
        terminal.present(&cells, width)?;
        std::thread::sleep(Duration::from_secs_f64(1. / 60.).saturating_sub(start.elapsed()));
        frame_ms = frame_ms * 0.9 + start.elapsed().as_secs_f32() * 1000. * 0.1;
    }
    Ok(())
}
