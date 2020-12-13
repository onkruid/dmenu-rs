use std::env;
use std::fs;

fn main() {
    let out_dir = env::current_dir().unwrap();

    // Write our version file
    let version = env!("CARGO_PKG_VERSION");
    let dest_path = out_dir.join("VERSION");
    fs::write(&dest_path, version).unwrap();
}
