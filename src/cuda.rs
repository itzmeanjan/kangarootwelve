use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CudaError {
    NoDevice,
    Allocation,
    Memcpy,
    KernelLaunch,
    Synchronization,
    GridDimensionExceeded,
    Unknown(i32),
}

impl CudaError {
    fn from_code(code: i32) -> Self {
        match code {
            1 => Self::NoDevice,
            2 => Self::Allocation,
            3 => Self::Memcpy,
            4 => Self::KernelLaunch,
            5 => Self::Synchronization,
            6 => Self::GridDimensionExceeded,
            other => Self::Unknown(other),
        }
    }
}

impl fmt::Display for CudaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::NoDevice => "no usable CUDA device found",
            Self::Allocation => "CUDA memory allocation failed",
            Self::Memcpy => "CUDA memory copy failed",
            Self::KernelLaunch => "CUDA kernel launch failed",
            Self::Synchronization => "CUDA synchronization failed",
            Self::GridDimensionExceeded => "launch grid exceeded device limits",
            Self::Unknown(code) => return write!(f, "unknown CUDA backend error (code {code})"),
        };

        f.write_str(msg)
    }
}

impl std::error::Error for CudaError {}

unsafe extern "C" {
    fn kt128_cuda_absorb(msg: *const u8, mlen: usize, cstr: *const u8, clen: usize, out_state: *mut u8) -> i32;
    fn kt256_cuda_absorb(msg: *const u8, mlen: usize, cstr: *const u8, clen: usize, out_state: *mut u8) -> i32;
}

type AbsorbFn = unsafe extern "C" fn(*const u8, usize, *const u8, usize, *mut u8) -> i32;

fn absorb_state(ffi: AbsorbFn, msg: &[u8], cstr: &[u8]) -> Result<[u64; 25], CudaError> {
    const KECCAK_WORD_BYTE_LENGTH: usize = turboshake::keccak::W / u8::BITS as usize;
    const KECCAK_STATE_BYTE_WIDTH: usize = turboshake::keccak::LANE_CNT * KECCAK_WORD_BYTE_LENGTH;

    let mut bytes = [0u8; KECCAK_STATE_BYTE_WIDTH];
    let rc = unsafe { ffi(msg.as_ptr(), msg.len(), cstr.as_ptr(), cstr.len(), bytes.as_mut_ptr()) };
    if rc != 0 {
        return Err(CudaError::from_code(rc));
    }

    let mut state = [0u64; turboshake::keccak::LANE_CNT];
    for (lane, chunk) in state.iter_mut().zip(bytes.chunks_exact(KECCAK_WORD_BYTE_LENGTH)) {
        *lane = u64::from_le_bytes(chunk.try_into().unwrap());
    }

    Ok(state)
}

pub(crate) fn kt128_absorb_state(msg: &[u8], cstr: &[u8]) -> Result<[u64; 25], CudaError> {
    absorb_state(kt128_cuda_absorb, msg, cstr)
}

pub(crate) fn kt256_absorb_state(msg: &[u8], cstr: &[u8]) -> Result<[u64; 25], CudaError> {
    absorb_state(kt256_cuda_absorb, msg, cstr)
}
