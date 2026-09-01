// code for static binary

use ascii_game::game_loop::*;

fn main() {
    let mut state = init_state();
    loop {
        fmain(&mut state)
    }
}
