use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", version);

    // let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = env::current_dir().unwrap();
    let dest_path = Path::new(&out_dir).join("VERSION");
    fs::write(
        &dest_path,
        version
    ).unwrap();
}
