use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_path = PathBuf::from(manifest_dir).join("obj");

    println!("cargo:rustc-link-search={}", lib_path.display());
    // println!("cargo:rerun-if-changed=libstaff-uart.a");
    println!("cargo:rerun-if-changed=kmalloc.o");
}