// code for hot reloading bin.
use ascii_game::init::*;
use ascii_game::state::*;

fn main() {
    let mut state = init_state();
    println!("Hot reloading is a work in progress!");
    unsafe {
        let mut lib: libloading::Library;
        let mut fun: libloading::Symbol<unsafe fn(*mut State) -> State>;
        (lib, fun) = load();
        loop {
            fun = reload(&mut lib);
            fun(&mut state);
        }
    }
}

unsafe fn load<'a>() -> (libloading::Library, libloading::Symbol<'a, unsafe fn(*mut State) -> State>) {
    todo!();
}

unsafe fn reload(lib: &mut libloading::Library) -> libloading::Symbol<unsafe fn(*mut State) -> State> {
    *lib = match libloading::Library::new("target/debug/libascii_game.so"){
        Ok(i) => i,
        Err(_) => panic!("Could not load library.")
    };
    match lib.get(b"fmain"){
        Ok(i) => i,
        Err(_) => panic!("Could not load `fmain` fn.")
    }
}
