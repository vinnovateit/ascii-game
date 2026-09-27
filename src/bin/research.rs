//! Reproducible experiment runner; intentionally separate from normal gameplay.
use ascii_game::{
    render::{
        ascii::Ascii,
        capture,
        framebuffer::Framebuffer,
        scene::{self, Mode},
    },
    research::{
        atlas::Atlas,
        matcher::{Matcher, Method, Settings},
    },
    viewpoints,
    world::test_scene::test_scene,
};
use std::{fs, io, path::PathBuf, time::Instant};
fn err(s: impl Into<String>) -> io::Error {
    io::Error::other(s.into())
}
fn main() -> io::Result<()> {
    let mut width = 120;
    let mut height = 40;
    let mut sw = 6;
    let mut sh = 10;
    let mut view = "doorway".to_string();
    let mut method = "gradient".to_string();
    let mut path = PathBuf::from("visual/research/trial");
    let mut atlas_path = None;
    let mut settings = Settings::default();
    let mut frames = 1usize;
    let mut motion = "static".to_string();
    let mut compact = false;
    let mut sequence = false;
    let mut ratio = 0.5;
    let mut gain = 1.;
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        if key == "--help" {
            println!(
                "research --method fast|mse|normalized|gradient|feature|scaled|offset --view doorway|crates|stairs --output DIR\n--width N --height N --sample-width N --sample-height N --atlas FILE --compact 0|1\n--frames N --motion static|yaw|forward|strafe|stairs --sequence 0|1\n--shortlist N --top-k N --tolerance F --hysteresis F --coherence F --gain F --cell-aspect F"
            );
            return Ok(());
        }
        let v = args.next().ok_or_else(|| err(format!("missing {key}")))?;
        let integer = || {
            v.parse::<usize>()
                .map_err(|_| err(format!("invalid {key}")))
        };
        let number = || v.parse::<f32>().map_err(|_| err(format!("invalid {key}")));
        match key.as_str() {
            "--width" => width = integer()?,
            "--height" => height = integer()?,
            "--sample-width" => sw = integer()?,
            "--sample-height" => sh = integer()?,
            "--view" => view = v,
            "--method" => method = v,
            "--output" => path = v.into(),
            "--atlas" => atlas_path = Some(PathBuf::from(v)),
            "--frames" => frames = integer()?,
            "--motion" => motion = v,
            "--sequence" => sequence = integer()? != 0,
            "--compact" => compact = integer()? != 0,
            "--shortlist" => settings.shortlist = integer()?,
            "--top-k" => settings.top_k = integer()?,
            "--tolerance" => settings.tolerance = number()?,
            "--hysteresis" => settings.hysteresis = number()?,
            "--coherence" => settings.coherence = number()?,
            "--gain" => gain = number()?,
            "--cell-aspect" => ratio = number()?,
            _ => return Err(err(format!("unknown {key}"))),
        }
    }
    if !(1..=500).contains(&width)
        || !(1..=240).contains(&height)
        || sw == 0
        || sh == 0
        || sw > 16
        || sh > 16
        || sw * sh > 128
        || frames == 0
        || frames > 10000
    {
        return Err(err("invalid dimensions or frame count"));
    }
    if ![
        gain,
        ratio,
        settings.tolerance,
        settings.hysteresis,
        settings.coherence,
    ]
    .iter()
    .all(|v| v.is_finite() && *v >= 0.)
        || !(0.2..=2.).contains(&ratio)
    {
        return Err(err("invalid numeric option"));
    }
    let mut camera =
        viewpoints::camera(&view, 70f32.to_radians()).ok_or_else(|| err("invalid view"))?;
    let origin = camera.position;
    let yaw = camera.yaw;
    let atlas = if let Some(p) = atlas_path {
        Atlas::load(&p, sw, sh, compact)?
    } else {
        Atlas::builtin(sw, sh, compact)
    };
    let scale = atlas.max_density;
    if method != "fast" {
        settings.method = Method::parse(&method).ok_or_else(|| err("invalid method"))?;
    }
    let mut matcher = Matcher::new(atlas, settings);
    let world = test_scene();
    let mut source = Framebuffer::new(width * sw, height * sh);
    let mut low = Framebuffer::new(width, height);
    let mut fast = Ascii::default();
    let mut cells = vec![b' '; width * height];
    let mut previous = cells.clone();
    let (mut render_ms, mut convert_ms, mut changes, mut visible) = (0., 0., 0usize, 0usize);
    fs::create_dir_all(&path)?;
    matcher.atlas.save(&path.join("atlas.pgm"))?;
    for frame in 0..frames {
        camera.position = origin;
        camera.yaw = yaw;
        let t = frame as f32 / 60.;
        match motion.as_str() {
            "static" => {}
            "yaw" => camera.yaw += t * 0.12,
            "forward" => camera.position += camera.forward() * t * 0.6,
            "strafe" => camera.position.x += t * 0.6,
            "stairs" => {
                camera.position.z -= t * 0.6;
                camera.position.y += t * 0.12;
            }
            _ => return Err(err("invalid motion")),
        }
        let start = Instant::now();
        if method == "fast" {
            scene::render(&world.meshes, &camera, &mut low, ratio, Mode::Ascii);
        } else {
            scene::render(
                &world.meshes,
                &camera,
                &mut source,
                ratio * sh as f32 / sw as f32,
                Mode::Ascii,
            );
        }
        render_ms += start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        if method == "fast" {
            fast.convert(&low, &camera, &mut cells, false);
        } else {
            for l in &mut source.luminance {
                *l = ((*l - 0.08) / 0.72).clamp(0., 1.) * scale * gain;
            }
            matcher.convert(&source, &mut cells);
        }
        convert_ms += start.elapsed().as_secs_f64() * 1000.;
        if frame > 0 {
            for (&a, &b) in previous.iter().zip(&cells) {
                if a != b {
                    changes += 1;
                }
                if a != b' ' || b != b' ' {
                    visible += 1;
                }
            }
        }
        previous.copy_from_slice(&cells);
        if sequence {
            save_text(&path.join(format!("frame-{frame:03}.txt")), &cells, width)?;
        }
    }
    // Capture raw high-resolution input separately from the timed conversion.
    let mut raw = Framebuffer::new(width * sw, height * sh);
    scene::render(
        &world.meshes,
        &camera,
        &mut raw,
        ratio * sh as f32 / sw as f32,
        Mode::Ascii,
    );
    capture::pgm(
        &path.join("raw.pgm"),
        raw.width,
        raw.height,
        raw.luminance.iter().copied(),
    )?;
    if method == "fast" {
        for (l, r) in source.luminance.iter_mut().zip(&raw.luminance) {
            *l = ((*r - 0.08) / 0.72).clamp(0., 1.) * scale * gain;
        }
    }
    capture::pgm(
        &path.join("source.pgm"),
        source.width,
        source.height,
        source.luminance.iter().copied(),
    )?;
    capture::pgm(
        &path.join("depth.pgm"),
        raw.width,
        raw.height,
        raw.depth.iter().map(|z| {
            let d = capture::linear_depth(*z, &camera);
            if d.is_finite() {
                1. - (d / camera.far).sqrt()
            } else {
                0.
            }
        }),
    )?;
    save_text(&path.join("frame.txt"), &cells, width)?;
    let reconstruction = matcher.reconstruct(&cells, width, false);
    capture::pgm(
        &path.join("reconstruction.pgm"),
        source.width,
        source.height,
        reconstruction.iter().copied(),
    )?;
    let (mse, mae, gradient) = errors(&source.luminance, &reconstruction, source.width);
    let mut counts = [0usize; 128];
    for c in &cells {
        counts[*c as usize] += 1;
    }
    let n = cells.len() as f32;
    let occupancy = 1. - counts[32] as f32 / n;
    let entropy = counts
        .iter()
        .filter(|v| **v > 0)
        .map(|v| {
            let p = *v as f32 / n;
            -p * p.log2()
        })
        .sum::<f32>();
    let density = cells
        .iter()
        .map(|c| {
            matcher
                .atlas
                .glyphs
                .iter()
                .find(|g| g.byte == *c)
                .map_or(0., |g| g.features.mean)
        })
        .sum::<f32>()
        / n;
    let mut stats = format!(
        "method={method}\nview={view}\nwidth={width}\nheight={height}\nsampling={sw}x{sh}\nframes={frames}\nmotion={motion}\nrender_ms={:.5}\nconvert_ms={:.5}\ncpu_ms={:.5}\noccupancy={occupancy:.6}\nentropy={entropy:.6}\natlas_density={density:.6}\nmse={mse:.6}\nmae={mae:.6}\ngradient_mae={gradient:.6}\nglyph_change_rate={:.6}\n",
        render_ms / frames as f64,
        convert_ms / frames as f64,
        (render_ms + convert_ms) / frames as f64,
        changes as f64 / visible.max(1) as f64
    );
    if method == "scaled" || method == "offset" {
        let r = matcher.reconstruct(&cells, width, true);
        capture::pgm(
            &path.join("intensity-reconstruction.pgm"),
            source.width,
            source.height,
            r.iter().copied(),
        )?;
        let (m, a, g) = errors(&source.luminance, &r, source.width);
        stats +=
            &format!("intensity_mse={m:.6}\nintensity_mae={a:.6}\nintensity_gradient_mae={g:.6}\n");
    }
    fs::write(
        path.join("distribution.txt"),
        counts
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(c, n)| format!("{:?} {n}\n", c as u8 as char))
            .collect::<String>(),
    )?;
    fs::write(path.join("metrics.txt"), &stats)?;
    println!("{} {}", path.display(), stats.replace('\n', " "));
    Ok(())
}
fn save_text(path: &std::path::Path, cells: &[u8], width: usize) -> io::Result<()> {
    let mut data = Vec::new();
    for row in cells.chunks(width) {
        data.extend_from_slice(row);
        data.push(b'\n');
    }
    fs::write(path, data)
}
fn errors(a: &[f32], b: &[f32], w: usize) -> (f32, f32, f32) {
    let n = a.len() as f32;
    let mut mse = 0.;
    let mut mae = 0.;
    let mut grad = 0.;
    for i in 0..a.len() {
        mse += (a[i] - b[i]).powi(2);
        mae += (a[i] - b[i]).abs();
        if i % w > 0 {
            grad += ((a[i] - a[i - 1]) - (b[i] - b[i - 1])).abs();
        }
        if i >= w {
            grad += ((a[i] - a[i - w]) - (b[i] - b[i - w])).abs();
        }
    }
    (mse / n, mae / n, grad / (2. * n))
}
