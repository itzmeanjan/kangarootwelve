#[cfg(all(feature = "cuda", feature = "multi_threaded"))]
compile_error!("features `cuda` and `multi_threaded` are mutually exclusive; enable at most one hashing backend");

#[cfg(feature = "cuda")]
mod cuda;

mod kt128;
mod kt256;
mod tests;

#[cfg(not(feature = "cuda"))]
mod utils;

#[cfg(feature = "cuda")]
pub use cuda::{CudaError, DeviceBuffer};

pub use kt128::{KT128, KT128XOF};
pub use kt256::{KT256, KT256XOF};
