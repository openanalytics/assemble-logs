extern crate built;
use built::Options;
use std::{env, path};

fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(
        Options::default().set_dependencies(true),
        src.as_ref(),
        &dst,
    )
    .expect("Failed to acquire build-time information");
}
