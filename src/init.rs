// this contains a simple function to init state struct.
// will be used by both hot reloader and static bin.

use crate::state::*;

pub fn init_state() -> State {
    return State {
        counter: 0,
    }
}
