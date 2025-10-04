mod consts;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod keccakx2;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod keccakx4;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod keccakx8;
