use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
#[derive(Default)]
pub struct Input {
    held: HashMap<KeyCode, Instant>,
    pub releases: bool,
}
impl Input {
    pub fn key(&mut self, key: KeyEvent, now: Instant) {
        if key.kind == KeyEventKind::Release {
            self.held.remove(&key.code);
        } else {
            self.held.insert(key.code, now);
        }
    }
    pub fn down(&self, code: KeyCode, now: Instant) -> bool {
        self.held
            .get(&code)
            .is_some_and(|t| self.releases || now.duration_since(*t) < Duration::from_millis(180))
    }
    pub fn clear(&mut self) {
        self.held.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;
    #[test]
    fn held_keys_release_and_legacy_keys_expire() {
        let mut input = Input::default();
        let now = Instant::now();
        input.key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE), now);
        assert!(input.down(KeyCode::Char('w'), now));
        assert!(!input.down(KeyCode::Char('w'), now + Duration::from_secs(1)));
        input.releases = true;
        assert!(input.down(KeyCode::Char('w'), now + Duration::from_secs(1)));
        input.key(
            KeyEvent::new_with_kind(
                KeyCode::Char('w'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            now,
        );
        assert!(!input.down(KeyCode::Char('w'), now));
    }
}
