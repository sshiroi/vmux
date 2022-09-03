extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(windows) {
        let inc_dir = env::var("LIBBLURAY_INCLUDE_DIR")
            .and_then(|inc_dir| match PathBuf::from(inc_dir) {
                inc_dir if inc_dir.is_dir() => Ok(inc_dir),
                _ => panic!("Bluray include problem"),
            })
            .expect("LIBBLURAY_INCLUDE_DIR is not set or the specified directory is not valid.");

        let libsdir = env::var("LIBBLURAY_AND_LIBS_DIR")
            .and_then(|inc_dir| match PathBuf::from(inc_dir) {
                inc_dir if inc_dir.is_dir() => Ok(inc_dir),
                _ => panic!("Bluray include problem"),
            })
            .expect("LIBBLURAY_AND_LIBS_DIR is not set or the specified directory is not valid.");

        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=bz2");
        println!("cargo:rustc-link-lib=dylib=libbluray");

        println!("cargo:rustc-link-search={}", libsdir.display().to_string());

        let bindings = bindgen::Builder::default()
            .clang_arg(format!("-I{}", inc_dir.as_os_str().to_str().unwrap()))
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    } else {
        let library = pkg_config::probe_library("libbluray").unwrap();
        println!("cargo:rustc-link-lib=bz2");
        println!("cargo:rerun-if-changed=wrapper.h");

        let bindings = bindgen::Builder::default()
            .clang_args(
                library
                    .include_paths
                    .iter()
                    .map(|path| format!("-I{}", path.to_string_lossy())),
            )
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
