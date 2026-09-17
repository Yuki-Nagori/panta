use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi_support.cc");
    println!("cargo:rerun-if-changed=include/panta/ffi.hpp");

    let mut builder = cxx_build::bridge("src/lib.rs");
    let cxx_standard_flag = if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        "/std:c++20"
    } else {
        "-std=c++20"
    };
    builder
        .file("src/ffi_support.cc")
        .include("include")
        .flag_if_supported(cxx_standard_flag)
        .compile("panta_ffi_bridge");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    let generated_header = find_generated_header(&out_dir).unwrap_or_else(|| {
        panic!(
            "cxx-build did not generate panta FFI header in {}",
            out_dir.display()
        )
    });
    let cxx_include = out_dir.join("cxxbridge").join("include");
    let public_header = cxx_include.join("panta_ffi.h");
    fs::copy(&generated_header, &public_header).unwrap_or_else(|error| {
        panic!(
            "copy generated CXX header {} to {}: {error}",
            generated_header.display(),
            public_header.display()
        )
    });

    // launcher/build.rs passes this directory and the staticlib to native CMake.
    println!("cargo:include={}", cxx_include.display());
}

fn find_generated_header(root: &Path) -> Option<PathBuf> {
    let mut directories = vec![root.to_owned()];
    while let Some(directory) = directories.pop() {
        let entries = fs::read_dir(&directory).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
            } else if path.file_name().is_some_and(|name| name == "lib.rs.h") {
                return Some(path);
            }
        }
    }
    None
}
