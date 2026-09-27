//! Interactive adapters for the two research finalists; fast remains independent.
use super::{
    atlas::Atlas,
    matcher::{Matcher, Settings},
    source::{SourcePass, SourceSettings},
};
use crate::{
    engine::camera::Camera,
    render::{
        ascii::Ascii,
        capture,
        framebuffer::Framebuffer,
        mesh::Mesh,
        scene::{self, Mode, Stats},
    },
};
use std::{io, path::Path};
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Backend {
    #[default]
    Fast,
    Matched,
    Enriched,
}
impl Backend {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "fast" => Ok(Self::Fast),
            "matched" => Ok(Self::Matched),
            "enriched" => Ok(Self::Enriched),
            _ => Err("ascii-backend must be fast, matched or enriched".into()),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Matched => "matched",
            Self::Enriched => "enriched",
        }
    }
}
pub fn samples(s: &str) -> Result<(usize, usize), String> {
    match s {
        "2x4" => Ok((2, 4)),
        "3x6" => Ok((3, 6)),
        "4x8" => Ok((4, 8)),
        "6x10" => Ok((6, 10)),
        "8x16" => Ok((8, 16)),
        _ => Err("match-samples must be 2x4, 3x6, 4x8, 6x10 or 8x16".into()),
    }
}
pub struct Experimental {
    kind: Backend,
    ratio: f32,
    pub source: Framebuffer,
    pass: SourcePass,
    pub matcher: Matcher,
}
impl Experimental {
    pub fn new(
        kind: Backend,
        sampling: (usize, usize),
        ratio: f32,
        font: Option<&Path>,
    ) -> io::Result<Self> {
        let atlas = if let Some(path) = font {
            Atlas::load(path, sampling.0, sampling.1, false)?
        } else {
            Atlas::builtin(sampling.0, sampling.1, false)
        };
        Ok(Self {
            kind,
            ratio,
            source: Framebuffer::new(1, 1),
            pass: SourcePass::default(),
            matcher: Matcher::new(
                atlas,
                Settings {
                    hysteresis: 0.035,
                    ..Settings::default()
                },
            ),
        })
    }
    pub fn render(
        &mut self,
        meshes: &[Mesh],
        camera: &Camera,
        low: &mut Framebuffer,
        fast: &mut Ascii,
        cells: &mut [u8],
    ) -> Stats {
        let w = self.matcher.atlas.width;
        let h = self.matcher.atlas.height;
        if self.source.width != low.width * w || self.source.height != low.height * h {
            self.source = Framebuffer::new(low.width * w, low.height * h);
            self.matcher.reset();
        }
        let ratio = self.ratio * h as f32 / w as f32;
        let stats = scene::render(meshes, camera, &mut self.source, ratio, Mode::Ascii);
        self.pass.apply(
            &mut self.source,
            camera,
            ratio,
            SourceSettings {
                noise: 0.22,
                lighting: true,
                ..SourceSettings::default()
            },
        );
        for cy in 0..low.height {
            for cx in 0..low.width {
                let mut light = 0.;
                let mut depth = f32::INFINITY;
                for y in 0..h {
                    for x in 0..w {
                        let j = (cy * h + y) * self.source.width + cx * w + x;
                        light += self.source.luminance[j];
                        depth = depth.min(self.source.depth[j]);
                    }
                }
                low.luminance[cy * low.width + cx] = light / (w * h) as f32;
                low.depth[cy * low.width + cx] = depth;
            }
        }
        if self.kind == Backend::Enriched {
            fast.convert(low, camera, cells, false);
        }
        let scale = self.matcher.atlas.max_density;
        for l in &mut self.source.luminance {
            *l = ((*l - 0.055) / 0.645).clamp(0., 1.) * scale;
        }
        if self.kind == Backend::Matched {
            self.matcher.convert(&self.source, cells);
        }
        stats
    }
    pub fn capture(&self, path: &Path, cells: &[u8], width: usize) -> io::Result<()> {
        self.matcher.atlas.save(&path.join("atlas.pgm"))?;
        capture::pgm(
            &path.join("source.pgm"),
            self.source.width,
            self.source.height,
            self.source.luminance.iter().copied(),
        )?;
        let reconstructed = self.matcher.reconstruct(cells, width, false);
        capture::pgm(
            &path.join("reconstruction.pgm"),
            self.source.width,
            self.source.height,
            reconstructed.into_iter(),
        )?;
        std::fs::write(
            path.join("backend.txt"),
            format!(
                "backend={}\nsamples={}x{}\nsource_noise=0.22\nlighting=true\ncontact=0\ncontext=0\nreconstruction_target_black=0.055\nreconstruction_target_white=0.7\n{}\nglyphs={:?}\n",
                self.kind.name(),
                self.matcher.atlas.width,
                self.matcher.atlas.height,
                if self.kind == Backend::Matched {
                    "metric=normalized+gradient\nsearch=exact density bound\nhysteresis=0.035"
                } else {
                    "mapper=accepted fast\nmapper_settings=style.txt\nhysteresis=0"
                },
                self.matcher
                    .atlas
                    .glyphs
                    .iter()
                    .map(|g| g.byte as char)
                    .collect::<String>()
            ),
        )
    }
}
