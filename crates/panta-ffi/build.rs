use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "../launcher/src/provision.rs"]
mod provision;

fn main() -> Result<(), io::Error> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi_support.cc");
    println!("cargo:rerun-if-changed=include/panta/ffi.hpp");
    println!("cargo:rerun-if-changed=../launcher/src/provision.rs");
    println!("cargo:rerun-if-env-changed=PANTA_USE_SYSTEM_TOOLS");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CLANG_FORMAT");

    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo 必须设置 OUT_DIR"))?,
    );
    let target_root = out_dir.ancestors().nth(4).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "无法从 OUT_DIR 推导 target 根")
    })?;
    let llvm = provision::resolve_llvm_compilers(target_root).map_err(io::Error::other)?;
    let compiler = match (
        env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc"),
        llvm.clang_cl.as_ref(),
    ) {
        (true, Some(path)) => path,
        _ => &llvm.clangxx,
    };

    let mut builder = cxx_build::bridge("src/lib.rs");
    let cxx_standard_flag = if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        "/std:c++20"
    } else {
        "-std=c++20"
    };
    builder
        .compiler(compiler)
        .file("src/ffi_support.cc")
        .include("include")
        .flag_if_supported(cxx_standard_flag)
        .compile("panta_ffi_bridge");

    let generated_header = find_generated_header(&out_dir).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "cxx-build did not generate panta FFI header in {}",
                out_dir.display()
            ),
        )
    })?;
    let cxx_include = out_dir.join("cxxbridge").join("include");
    let public_header = cxx_include.join("panta_ffi.h");
    fs::copy(&generated_header, &public_header).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "copy generated CXX header {} to {}: {error}",
                generated_header.display(),
                public_header.display()
            ),
        )
    })?;

    // launcher/build.rs passes this directory and the staticlib to native CMake.
    println!("cargo:include={}", cxx_include.display());
    Ok(())
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
