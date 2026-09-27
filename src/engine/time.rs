pub const STEP: f32 = 1. / 120.;
#[derive(Default)]
pub struct FixedClock {
    accumulator: f32,
}
impl FixedClock {
    pub fn advance(&mut self, elapsed: f32) -> usize {
        self.accumulator += elapsed.clamp(0., 0.1);
        let steps = (self.accumulator / STEP) as usize;
        self.accumulator -= steps as f32 * STEP;
        steps
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn update_rate_independent_of_render_rate() {
        let count = |hz: usize| {
            let mut c = super::FixedClock::default();
            (0..hz).map(|_| c.advance(1. / hz as f32)).sum::<usize>()
        };
        assert_eq!(count(30), 120);
        assert_eq!(count(60), 120);
    }
}
