// code for hot reloading bin.
use ascii_game::init::*;
use ascii_game::state::*;
use libloading::*;

fn main() {
    let mut state = init_state();
    println!("Hot reloading is a work in progress!");
    unsafe {
        let mut lib = match Library::new("target/debug/libascii_game.so"){
            Ok(i) => i,
            Err(_) => panic!("Could not load library.")
        };
        loop {
            Library::close(lib);
            lib = match Library::new("target/debug/libascii_game.so"){
                Ok(i) => i,
                Err(_) => panic!("Could not load library.")
            };
            let mut fun: Symbol<unsafe fn(*mut State) -> State> = match lib.get(b"fmain"){
                Ok(i) => i,
                Err(_) => panic!("Could not load `fmain` fn.")
            };
            fun(&mut state);
        }
    }
}
