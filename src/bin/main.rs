// code for static binary

use ascii_game::game_loop::*;
use ascii_game::state::*;

fn main() {
    let mut state = init_state();
    loop {
        fmain(&mut state)
    }
}
