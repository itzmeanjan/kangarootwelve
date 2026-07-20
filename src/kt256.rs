#[cfg(not(feature = "cuda"))]
use crate::utils::{get_ith_chunk, length_encode};
use turboshake::sponge;

#[cfg(feature = "cuda")]
use crate::cuda::DeviceBuffer;
#[cfg(feature = "cuda")]
use core::time::Duration;

#[cfg(not(feature = "cuda"))]
use turboshake::TurboShake256;

#[cfg(feature = "multi_threaded")]
use std::cmp;

#[cfg(feature = "multi_threaded")]
use rayon::{ThreadPoolBuilder, prelude::*};

/// KT256 Extendable Output Function (XOF).
/// It allows absorbing arbitrary long input message, with optional domain separation using a customization string.
/// It only supports absorbing the full message in a single call, i.e., it does not allow streamed hashing.
/// The returned object can be squeezed for producing arbitrary long output.
/// When absorbing a long message the absorption phase can benefit from multi-threading - it is feature-gated behind `multi_threaded`.
/// Optionally one might want to hash on GPU, exploiting its high degree of parallelism - it is feature-gated behind `cuda`.
/// Hashing only on NVIDIA GPU is supported for now.
/// It supports both hashing GPU memory-resident data and transferring data to GPU, followed by hashing.
///
/// RFC 9861 specifies KangarooTwelve <https://www.rfc-editor.org/rfc/rfc9861.html>.
pub struct KT256;

/// KT256 Extendable Output Function (XOF) Squeezer.
/// This is the return type of the [KT256] hasher.
/// It allows squeezing arbitrary long output from the sponge.
/// Squeezing always runs on CPU, single-threaded.
#[derive(Copy, Clone)]
pub struct KT256XOF {
    state: [u64; turboshake::keccak::LANE_CNT],
    squeezable: usize,
}

impl KT256 {
    const TARGET_BIT_SECURITY: usize = 256;
    const CAPACITY_BITS: usize = 2 * Self::TARGET_BIT_SECURITY;
    const KECCAK_STATE_BIT_WIDTH: usize = turboshake::keccak::LANE_CNT * turboshake::keccak::W;
    const RATE_BITS: usize = Self::KECCAK_STATE_BIT_WIDTH - Self::CAPACITY_BITS;
    const RATE_BYTES: usize = Self::RATE_BITS / u8::BITS as usize;

    #[cfg(not(feature = "cuda"))]
    const CHUNK_BYTE_LEN: usize = 8192;
    #[cfg(not(feature = "cuda"))]
    const D_SEP_A: u8 = 0x07;
    #[cfg(not(feature = "cuda"))]
    const D_SEP_B: u8 = 0x0b;
    #[cfg(not(feature = "cuda"))]
    const D_SEP_C: u8 = 0x06;

    /// Given a messsage M and a customization string C, this routine absorbs them into the Keccak\[512\] sponge.
    /// Both M and C can be arbitrary long.
    /// C can be used for domain separation.
    /// Absorption is implemented to be single-threaded.
    /// The returned object can be used for squeezing arbitrary long output.
    #[cfg(not(any(feature = "cuda", feature = "multi_threaded")))]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> KT256XOF {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = tlen.div_ceil(Self::CHUNK_BYTE_LEN);

        if n == 1 {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, clen) = get_ith_chunk::<{ Self::CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen]);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_A }>(&mut state, &mut offset);

            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, _) = get_ith_chunk::<{ Self::CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen]);
            const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
            const PAD_B: [u8; 2] = [0xff, 0xff];

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &PAD_A);

            for i in 1..n {
                let (chunk, clen) = get_ith_chunk::<{ Self::CHUNK_BYTE_LEN }>(i, msg, cstr, &enc[..elen]);
                let mut cv = [0u8; 64];

                unsafe {
                    let mut hasher = TurboShake256::default();
                    hasher.absorb(&chunk[..clen]).unwrap_unchecked();
                    hasher.finalize::<{ Self::D_SEP_B }>().unwrap_unchecked();
                    hasher.squeeze(&mut cv).unwrap_unchecked();
                }

                sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &cv);
            }

            let (enc, elen) = length_encode(n - 1);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &enc[..elen]);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &PAD_B);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_C }>(&mut state, &mut offset);

            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            }
        }
    }

    /// Given a messsage M and a customization string C, this routine absorbs them into the Keccak\[512\] sponge.
    /// Both M and C can be arbitrary long.
    /// C can be used for domain separation.
    /// Message absorption is multi-threaded on CPU systems.
    /// It launches N-many threads such that N is the number of logical threads on the execution environment.
    /// The returned object can be used for squeezing arbitrary long output.
    #[cfg(feature = "multi_threaded")]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> KT256XOF {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = tlen.div_ceil(Self::CHUNK_BYTE_LEN);

        if n == 1 {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, clen) = get_ith_chunk::<{ Self::CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen]);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_A }>(&mut state, &mut offset);

            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, _) = get_ith_chunk::<{ Self::CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen]);
            const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
            const PAD_B: [u8; 2] = [0xff, 0xff];

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &PAD_A);

            let cpus = cmp::min(num_cpus::get(), n - 1);
            let pool = ThreadPoolBuilder::new().num_threads(cpus).build().unwrap();
            let cvs = pool.install(|| {
                let mut cvs = vec![0u8; (n - 1) * 64];

                cvs.par_chunks_mut(64).enumerate().for_each(|(i, cv)| {
                    let (chunk, clen) = get_ith_chunk::<{ Self::CHUNK_BYTE_LEN }>(i + 1, msg, cstr, &enc[..elen]);

                    unsafe {
                        let mut hasher = TurboShake256::default();
                        hasher.absorb(&chunk[..clen]).unwrap_unchecked();
                        hasher.finalize::<{ Self::D_SEP_B }>().unwrap_unchecked();
                        hasher.squeeze(cv).unwrap_unchecked();
                    }
                });

                cvs
            });

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &cvs);

            let (enc, elen) = length_encode(n - 1);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &enc[..elen]);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &PAD_B);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_C }>(&mut state, &mut offset);

            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            }
        }
    }

    /// Given a messsage M and a customization string C, this routine absorbs them into the Keccak\[512\] sponge.
    /// Both M and C can be arbitrary long.
    /// C can be used for domain separation.
    /// It transfers both M and C to GPU memory, followed by absorbing them on GPU, benefiting from its single instruction multiple threads (SIMT) parallelism.
    /// For now it supports only NVIDIA GPUs.
    /// The returned object can be used for squeezing arbitrary long output.
    #[cfg(feature = "cuda")]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> KT256XOF {
        let dmsg = DeviceBuffer::new(msg).unwrap_or_else(|e| panic!("KT256 GPU hashing failed: {e}"));
        Self::hash_device(&dmsg, cstr).0
    }

    /// Given a messsage M and a customization string C, this routine absorbs them into the Keccak\[512\] sponge.
    /// Both M and C can be arbitrary long.
    /// C can be used for domain separation.
    /// M should already be GPU memory-resident, while C is host memory-resident.
    /// It allows hashing data at high-throughput, when the data is already present on GPU.
    /// For now it supports only NVIDIA GPUs.
    /// The returned object can be used for squeezing arbitrary long output.
    #[cfg(feature = "cuda")]
    pub fn hash_device(dmsg: &DeviceBuffer, cstr: &[u8]) -> (KT256XOF, Duration) {
        let (state, elapsed) = crate::cuda::kt256_absorb_device(dmsg, cstr).unwrap_or_else(|e| panic!("KT256 GPU hashing failed: {e}"));

        (
            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            },
            elapsed,
        )
    }
}

impl KT256XOF {
    /// Squeezes arbitrary long output from the KT256 sponge.
    /// Safe to call as many times needed.
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        sponge::squeeze::<{ KT256::RATE_BYTES }>(&mut self.state, &mut self.squeezable, out);
    }
}
