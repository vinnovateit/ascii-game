pub struct State {
    // add state which is to be preserved on hot reloading here.
    pub counter: i32
}

pub fn init_state() -> State {
    return State {
        counter: 0,
    }
}
