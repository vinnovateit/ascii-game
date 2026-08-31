// code for hot reloading bin.
use ascii_game::state::*;
use ascii_game::init::*;
use ascii_game::game_loop::*;

fn main() {
    let mut state = init_state();
    println!("Hot reloading is a work in progress!");
    loop {
        fmain(&mut state)
    }
}
