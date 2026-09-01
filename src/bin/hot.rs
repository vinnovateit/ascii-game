// code for hot reloading bin.
use ascii_game::state::*;

use libloading::*;
use std::time::SystemTime;
use std::fs;

const LIB_PATH: &str = "target/debug/libascii_game.so";

fn main() {
    let mut state = init_state();
    println!("Hot reloading is a work in progress!");

    unsafe {
        let mut lib = match Library::new(LIB_PATH){
            Ok(i) => i,
            Err(_) => panic!("Could not load library.")
        };

        let mut last_modified_when_loaded: SystemTime = SystemTime::now();

        loop {
            
            let mut fun: Symbol<unsafe fn(*mut State) -> State> = match lib.get(b"fmain"){
                Ok(i) => i,
                Err(_) => panic!("Could not load `fmain` fn.")
            };

            let last_modified: SystemTime = match fs::metadata(LIB_PATH){
                Ok(i) => fs::Metadata::modified(&i).expect("Could not get last modified time."),
                Err(_) => panic!("Could not read file metadata.")
            };

            let mut reload: bool = last_modified > last_modified_when_loaded;

            while reload {
                println!("Attempting to reload library.");
                _ = Library::close(lib);
                lib = match Library::new(LIB_PATH){
                    Ok(i) => {
                        i
                    }
                    Err(_) => panic!("Could not load library.")
                };
                fun = match lib.get(b"fmain"){
                    Ok(i) => {
                        reload = false;
                        i
                    }
                    Err(_) => panic!("Could not load `fmain` fn.")
                };
                last_modified_when_loaded = last_modified;
            }

            fun(&mut state);
        }
    }
}
