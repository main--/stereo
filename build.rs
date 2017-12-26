extern crate pkg_config;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=glib-2.0");

    let mono = pkg_config::probe_library("mono-2").unwrap();

    let bindings = bindgen::Builder::default()
        .clang_args(mono.include_paths.iter().map(|p| "-I".to_owned() + p.to_str().unwrap()))
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

