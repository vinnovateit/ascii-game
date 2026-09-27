/// Stable, versioned payload shared with the experimental dynamic library.
#[repr(C)]
#[derive(Default)]
pub struct State {
    pub counter: i32,
}
pub fn init_state() -> State {
    State::default()
}
