use std::env;
use std::fs;

fn main() {
    let out_dir = env::current_dir().unwrap();

    // Write our version file
    let version = env!("CARGO_PKG_VERSION");
    let dest_path = out_dir.join("VERSION");
    fs::write(&dest_path, version).unwrap();

    // Plugins
    let fuzzy = env::var("CARGO_FEATURE_FUZZY").is_ok();
    if fuzzy {
        dbg!("FUZZY");
    } else {
        dbg!("NOT FUZZY");
    }
}
