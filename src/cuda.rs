use core::{fmt, time::Duration};

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
    fn kt128_cuda_absorb_device(dmsg: *const u8, mlen: usize, cstr: *const u8, clen: usize, out_state: *mut u8, out_elapsed_ns: *mut u64) -> i32;
    fn kt256_cuda_absorb_device(dmsg: *const u8, mlen: usize, cstr: *const u8, clen: usize, out_state: *mut u8, out_elapsed_ns: *mut u64) -> i32;
    fn kt_cuda_upload(src: *const u8, len: usize) -> *mut u8;
    fn kt_cuda_release(dptr: *mut u8);
}

/// An owned region of CUDA device memory holding a byte buffer uploaded from host (CPU) memory.
/// Dropping the buffer frees the underlying device allocation.
pub struct DeviceBuffer {
    ptr: *mut u8,
    len: usize,
}

impl DeviceBuffer {
    pub fn new(data: &[u8]) -> Result<Self, CudaError> {
        if data.is_empty() {
            return Ok(Self {
                ptr: core::ptr::null_mut(),
                len: 0,
            });
        }

        let ptr = unsafe { kt_cuda_upload(data.as_ptr(), data.len()) };
        if ptr.is_null() {
            return Err(CudaError::Allocation);
        }

        Ok(Self { ptr, len: data.len() })
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        unsafe { kt_cuda_release(self.ptr) };
    }
}

type AbsorbFn = unsafe extern "C" fn(*const u8, usize, *const u8, usize, *mut u8, *mut u64) -> i32;

fn absorb_device(ffi: AbsorbFn, dmsg: &DeviceBuffer, cstr: &[u8]) -> Result<([u64; 25], Duration), CudaError> {
    const KECCAK_WORD_BYTE_LENGTH: usize = turboshake::keccak::W / u8::BITS as usize;
    const KECCAK_STATE_BYTE_WIDTH: usize = turboshake::keccak::LANE_CNT * KECCAK_WORD_BYTE_LENGTH;

    let mut bytes = [0u8; KECCAK_STATE_BYTE_WIDTH];
    let mut elapsed_ns = 0u64;

    let rc = unsafe { ffi(dmsg.as_ptr(), dmsg.len(), cstr.as_ptr(), cstr.len(), bytes.as_mut_ptr(), &mut elapsed_ns) };
    if rc != 0 {
        return Err(CudaError::from_code(rc));
    }

    let mut state = [0u64; turboshake::keccak::LANE_CNT];
    for (lane, chunk) in state.iter_mut().zip(bytes.chunks_exact(KECCAK_WORD_BYTE_LENGTH)) {
        *lane = u64::from_le_bytes(chunk.try_into().unwrap());
    }

    Ok((state, Duration::from_nanos(elapsed_ns)))
}

pub(crate) fn kt128_absorb_device(dmsg: &DeviceBuffer, cstr: &[u8]) -> Result<([u64; 25], Duration), CudaError> {
    absorb_device(kt128_cuda_absorb_device, dmsg, cstr)
}

pub(crate) fn kt256_absorb_device(dmsg: &DeviceBuffer, cstr: &[u8]) -> Result<([u64; 25], Duration), CudaError> {
    absorb_device(kt256_cuda_absorb_device, dmsg, cstr)
}
