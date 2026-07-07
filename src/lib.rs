#[cfg(all(feature = "cuda", feature = "multi_threaded"))]
compile_error!("features `cuda` and `multi_threaded` are mutually exclusive; enable at most one hashing backend");

#[cfg(feature = "cuda")]
mod cuda;

mod kt128;
mod kt256;
mod tests;
mod utils;

pub use kt128::{KT128, KT128XOF};
pub use kt256::{KT256, KT256XOF};
