pub mod consts;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod cvx2;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod cvx4;
