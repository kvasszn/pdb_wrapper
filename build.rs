use bindgen::CargoCallbacks;
use cmake::Config;
use std::{env, path::PathBuf};
use std::process::Command;
    use which::which;

#[cfg(all(feature = "llvm_10", feature = "llvm_22"))]
compile_error!("You may only enable one LLVM version");

#[cfg(feature = "llvm_10")]
const LLVM_VERSION: u32 = 10;

#[cfg(feature = "llvm_22")]
const LLVM_VERSION: u32 = 22;

fn version_specific_init() {
    // TODO: Does this work as expected on Windows?
    let binary = which("llvm-config")
        .expect("llvm-config was not found, please make sure that it's contained in your PATH!");

    // Get required libraries from `llvm-config`
    let result = Command::new(binary)
        .arg("--libnames")
        .arg("DebugInfoPDB")
        .output()
        .expect("Failed to run `llvm-config`");

    let result = String::from_utf8(result.stdout).expect("Failed to parse `llvm-config` output!");
    if cfg!(unix) {
        result
            .trim()
            .replace(".a", "")
            .replace(".so", "")
            .split_whitespace()
            .map(|lib| lib.trim())
            .filter_map(|lib| lib.strip_prefix("lib"))
            .filter(|lib| !lib.is_empty())
            .for_each(|lib| {
                println!("cargo:rustc-link-lib={}", lib);
            });
    } else if cfg!(windows) {
        result
            .trim()
            .replace(".lib", "")
            .split_whitespace()
            .map(|lib| lib.trim())
            .filter(|lib| !lib.is_empty())
            .for_each(|lib| {
                println!("cargo:rustc-link-lib={}", lib);
            });
    }

}

fn main() {
    let dst = Config::new("libllvm-pdb-wrapper").build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rerun-if-changed={}", dst.display());

    println!("cargo:rustc-link-lib=static=llvm-pdb-wrapper");
    println!("cargo:rustc-link-search=native=C:\\Program Files\\LLVM\\lib");
    println!("cargo:rustc-link-lib=llvm-pdb-wrapper");

    version_specific_init();

    if cfg!(unix) {
        println!("cargo:rustc-link-lib=ncurses");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=stdc++");
    } else if cfg!(windows) {
        //println!("cargo:rustc-link-lib=zlib");
    }

    println!("cargo:rerun-if-changed=libllvm-pdb-wrapper/wrapper.hpp");
    println!("cargo:rerun-if-changed=libllvm-pdb-wrapper/wrapper.cpp");

    let bindings = bindgen::Builder::default()
        .header("libllvm-pdb-wrapper/wrapper.hpp")
        .clang_arg("-x").clang_arg("c++")
        .clang_arg("-std=c++17")
        .clang_arg(format!("-DLLVM_VERSION_MAJOR={}", LLVM_VERSION))
        .allowlist_function("PDB_File_.*")
        .parse_callbacks(Box::new(CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
