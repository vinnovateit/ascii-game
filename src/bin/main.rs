fn main() {
    let mut state = ascii_game::state::init_state();
    loop {
        unsafe {
            ascii_game::game_loop::fmain(&mut state);
        }
        println!("Counter: {}", state.counter);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
