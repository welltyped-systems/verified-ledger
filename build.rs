use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=lean/VerifiedLedger/Model.lean");
    println!("cargo:rerun-if-changed=lean/VerifiedLedger/FFI.lean");
    println!("cargo:rerun-if-changed=lean/VerifiedLedger.lean");
    println!("cargo:rerun-if-changed=lean/lakefile.lean");
    println!("cargo:rerun-if-changed=src/lean_wrapper.c");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let lean_dir = manifest_dir.join("lean");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("out dir"));
    let ffi_c = out_dir.join("verified_ledger_ffi.c");

    run_lake_lean(&lean_dir, &ffi_c);

    let lean_prefix = lean_prefix(&lean_dir);
    let include_dir = Path::new(&lean_prefix).join("include");
    let lib_dir = Path::new(&lean_prefix).join("lib").join("lean");
    let toolchain_lib_dir = Path::new(&lean_prefix).join("lib");

    let model_c = lean_dir
        .join(".lake")
        .join("build")
        .join("ir")
        .join("VerifiedLedger")
        .join("Model.c");

    cc::Build::new()
        .file(&ffi_c)
        .file(model_c)
        .file(manifest_dir.join("src").join("lean_wrapper.c"))
        .include(include_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-label")
        .compile("verified_ledger_lean");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        toolchain_lib_dir.display()
    );
    println!("cargo:rustc-link-lib=static=Init");
    println!("cargo:rustc-link-lib=static=Std");
    println!("cargo:rustc-link-lib=static=leanrt");
    println!("cargo:rustc-link-lib=static=gmp");
    println!("cargo:rustc-link-lib=static=uv");
    println!("cargo:rustc-link-lib=dylib=c++");
}

fn run_lake_lean(lean_dir: &Path, output: &Path) {
    let status = Command::new("lake")
        .current_dir(lean_dir)
        .arg("lean")
        .arg("VerifiedLedger/FFI.lean")
        .arg("--")
        .arg("-c")
        .arg(output)
        .status()
        .expect("failed to run lake");

    if !status.success() {
        panic!("lake lean failed with status {}", status);
    }
}

fn lean_prefix(lean_dir: &Path) -> String {
    let output = Command::new("lake")
        .current_dir(lean_dir)
        .arg("env")
        .arg("lean")
        .arg("--print-prefix")
        .output()
        .expect("failed to query lean prefix");

    if !output.status.success() {
        panic!("lean --print-prefix failed");
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
