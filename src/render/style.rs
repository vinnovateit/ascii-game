#[derive(Clone, Debug, PartialEq)]
pub struct AsciiStyle {
    pub exposure: f32,
    pub gamma: f32,
    pub contrast: f32,
    pub black_point: f32,
    pub white_point: f32,
    pub background_cutoff: f32,
    pub posterization_levels: usize,
    pub dither_strength: f32,
    pub texture_variance: f32,
    pub depth_falloff: f32,
    pub edge_weight: f32,
    pub edge_threshold: f32,
}
impl Default for AsciiStyle {
    fn default() -> Self {
        Self::preset("pixatool").unwrap()
    }
}
impl AsciiStyle {
    pub fn preset(name: &str) -> Option<Self> {
        Some(match name {
            "pixatool" => Self {
                exposure: 1.05,
                gamma: 0.95,
                contrast: 1.6,
                black_point: 0.16,
                white_point: 0.82,
                background_cutoff: 0.23,
                posterization_levels: 6,
                dither_strength: 0.12,
                texture_variance: 0.35,
                depth_falloff: 0.065,
                edge_weight: 0.15,
                edge_threshold: 0.1,
            },
            "neutral" => Self {
                exposure: 1.,
                gamma: 1.,
                contrast: 1.,
                black_point: 0.,
                white_point: 1.,
                background_cutoff: 0.08,
                posterization_levels: 8,
                dither_strength: 0.,
                texture_variance: 0.,
                depth_falloff: 0.,
                edge_weight: 0.,
                edge_threshold: 0.1,
            },
            _ => return None,
        })
    }
    pub fn value(&self, index: usize) -> f32 {
        match index {
            0 => self.exposure,
            1 => self.gamma,
            2 => self.contrast,
            3 => self.black_point,
            4 => self.white_point,
            5 => self.background_cutoff,
            6 => self.posterization_levels as f32,
            7 => self.dither_strength,
            8 => self.texture_variance,
            9 => self.depth_falloff,
            10 => self.edge_weight,
            _ => self.edge_threshold,
        }
    }
    fn assign(&mut self, index: usize, value: f32) {
        match index {
            0 => self.exposure = value,
            1 => self.gamma = value,
            2 => self.contrast = value,
            3 => self.black_point = value,
            4 => self.white_point = value,
            5 => self.background_cutoff = value,
            6 => self.posterization_levels = value as usize,
            7 => self.dither_strength = value,
            8 => self.texture_variance = value,
            9 => self.depth_falloff = value,
            10 => self.edge_weight = value,
            _ => self.edge_threshold = value,
        }
    }
    pub fn set(&mut self, index: usize, value: f32) -> Result<(), String> {
        let p = &PARAMETERS[index];
        if !value.is_finite()
            || !(p.min..=p.max).contains(&value)
            || (index == 6 && value.fract() != 0.)
        {
            return Err(format!(
                "{} must be {}..{}{}",
                p.name,
                p.min,
                p.max,
                if index == 6 { " (integer)" } else { "" }
            ));
        }
        self.assign(index, value);
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.white_point - self.black_point < 0.02 - 1e-6 {
            return Err("white-point must exceed black-point by at least 0.02".into());
        }
        Ok(())
    }
    pub fn adjust(&mut self, index: usize, direction: f32, coarse: bool) {
        let p = &PARAMETERS[index];
        let mut v = (self.value(index) + direction * p.step * if coarse { 5. } else { 1. })
            .clamp(p.min, p.max);
        if index == 3 {
            v = v.min(self.white_point - 0.02);
        }
        if index == 4 {
            v = v.max(self.black_point + 0.02);
        }
        self.assign(index, v);
    }
}
pub struct Parameter {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}
pub const PARAMETERS: [Parameter; 12] = [
    Parameter {
        name: "exposure",
        min: 0.1,
        max: 4.,
        step: 0.05,
    },
    Parameter {
        name: "gamma",
        min: 0.1,
        max: 4.,
        step: 0.05,
    },
    Parameter {
        name: "contrast",
        min: 0.1,
        max: 4.,
        step: 0.05,
    },
    Parameter {
        name: "black-point",
        min: 0.,
        max: 0.98,
        step: 0.01,
    },
    Parameter {
        name: "white-point",
        min: 0.02,
        max: 1.,
        step: 0.01,
    },
    Parameter {
        name: "background-cutoff",
        min: 0.,
        max: 1.,
        step: 0.01,
    },
    Parameter {
        name: "posterization-levels",
        min: 4.,
        max: 12.,
        step: 1.,
    },
    Parameter {
        name: "dither-strength",
        min: 0.,
        max: 2.,
        step: 0.05,
    },
    Parameter {
        name: "texture-variance",
        min: 0.,
        max: 1.,
        step: 0.05,
    },
    Parameter {
        name: "depth-falloff",
        min: 0.,
        max: 0.5,
        step: 0.005,
    },
    Parameter {
        name: "edge-weight",
        min: 0.,
        max: 1.,
        step: 0.05,
    },
    Parameter {
        name: "edge-threshold",
        min: 0.01,
        max: 1.,
        step: 0.01,
    },
];
pub fn parameter(name: &str) -> Option<usize> {
    PARAMETERS
        .iter()
        .position(|p| p.name == name)
        .or(match name {
            "levels" => Some(6),
            "dither" => Some(7),
            _ => None,
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_values_and_point_order_rejected() {
        let mut s = AsciiStyle::default();
        assert!(s.set(1, f32::NAN).is_err());
        assert!(s.set(6, 5.5).is_err());
        assert!(s.set(6, 13.).is_err());
        s.black_point = 0.9;
        assert!(s.validate().is_err());
    }
    #[test]
    fn tuning_stays_in_range_and_preserves_point_order() {
        for index in 0..12 {
            let mut s = AsciiStyle::default();
            for _ in 0..100 {
                s.adjust(index, 1., true);
                assert!(s.validate().is_ok());
            }
            for _ in 0..100 {
                s.adjust(index, -1., true);
                assert!(s.validate().is_ok());
            }
        }
    }
}
