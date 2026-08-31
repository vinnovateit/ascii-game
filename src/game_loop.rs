use std::{thread, time};

use crate::state::*;

#[unsafe(no_mangle)]
pub fn fmain(state: &mut State) {
    println!("This is the main game loop: {}", state.counter);
    state.counter += 1;
    // sleeping to simulate work.
    // this will be replaced by game logic that runs on each frame.
    thread::sleep(time::Duration::from_secs(1));
}
