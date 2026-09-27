//! Counter-only ABI experiment; the renderer owns its Rust state statically.
use ascii_game::{game_loop::ABI_VERSION, state::init_state};
use libloading::Library;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
struct Loaded {
    library: Library,
    path: PathBuf,
}
impl Loaded {
    fn open(source: &Path, generation: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let path =
            source.with_file_name(format!(".ascii-hot-{}-{generation}.so", std::process::id()));
        fs::copy(source, &path)?;
        let result = (|| {
            let library = unsafe { Library::new(&path)? };
            let version =
                unsafe { library.get::<unsafe extern "C" fn() -> u32>(b"ascii_abi_version")? };
            if unsafe { version() } != ABI_VERSION {
                return Err("incompatible hot ABI".into());
            }
            unsafe {
                library.get::<unsafe extern "C" fn(*mut ascii_game::state::State)>(b"fmain")?;
            }
            Ok(Self {
                library,
                path: path.clone(),
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(path);
        }
        result
    }
}
impl Drop for Loaded {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/debug/libascii_game.so".into());
    let source = Path::new(&source);
    let mut generation = 0;
    let mut loaded = Loaded::open(source, generation)?;
    let mut modified = fs::metadata(source)?.modified()?;
    let mut state = init_state();
    loop {
        let next = fs::metadata(source)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if next > modified {
            generation += 1;
            match Loaded::open(source, generation) {
                Ok(candidate) => {
                    loaded = candidate;
                    modified = next;
                    eprintln!("Reloaded ABI v{ABI_VERSION}");
                }
                Err(error) => eprintln!("Keeping previous library: {error}"),
            }
        }
        unsafe {
            loaded
                .library
                .get::<unsafe extern "C" fn(*mut ascii_game::state::State)>(b"fmain")?(
                &mut state
            );
        }
        println!("Hot counter: {}", state.counter);
        std::thread::sleep(Duration::from_secs(1));
    }
}
