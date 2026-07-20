use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_CUDA");

    if env::var_os("CARGO_FEATURE_CUDA").is_none() {
        return;
    }

    println!("cargo:rerun-if-env-changed=NVCC");
    println!("cargo:rerun-if-env-changed=KT_CUDA_ARCH");
    println!("cargo:rerun-if-env-changed=CUDA_LIB_DIR");

    for f in ["cuda/ffi.cu", "cuda/kangarootwelve.cuh", "cuda/keccak.cuh", "cuda/sponge.cuh", "cuda/utils.cuh"] {
        println!("cargo:rerun-if-changed={f}");
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".to_string());
    let arch = env::var("KT_CUDA_ARCH").unwrap_or_else(|_| "sm_80".to_string());
    let cuda_lib_dir = env::var("CUDA_LIB_DIR").unwrap_or_else(|_| "/usr/local/cuda/lib64".to_string());

    let lib_path = out_dir.join("libkt_gpu.a");

    let output = Command::new(&nvcc)
        .args(["-O3", "-std=c++20"])
        .arg(format!("-arch={arch}"))
        .arg("-lib")
        .arg("cuda/ffi.cu")
        .arg("-o")
        .arg(&lib_path)
        .output()
        .unwrap_or_else(|e| panic!("failed to run nvcc ({nvcc}) — is the CUDA toolkit installed and on PATH? {e}"));

    if !output.status.success() {
        panic!("nvcc failed to compile the CUDA backend:\n{}", String::from_utf8_lossy(&output.stderr));
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=kt_gpu");
    println!("cargo:rustc-link-search=native={cuda_lib_dir}");
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
