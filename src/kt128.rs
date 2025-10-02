use crate::{cv::consts::CHUNK_BYTE_LEN, utils::length_encode};
use std::cmp;
use turboshake::{TurboShake128, keccak, sponge};

#[cfg(feature = "multi_threaded")]
use rayon::{ThreadPoolBuilder, prelude::*};

/// KT128 Extendable Output Function (XOF)
///
/// See <https://keccak.team/files/KangarooTwelve.pdf> and <https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve>
#[derive(Copy, Clone)]
pub struct KT128 {
    state: [u64; keccak::LANE_CNT],
    is_ready: usize,
    squeezable: usize,
}

impl KT128 {
    const KECCAK_STATE_BIT_LEN: usize = keccak::W * keccak::LANE_CNT;
    const BIT_SECURITY: usize = 128;

    const CAPACITY_BITS: usize = 2 * Self::BIT_SECURITY;
    const RATE_BITS: usize = Self::KECCAK_STATE_BIT_LEN - Self::CAPACITY_BITS;
    const RATE_BYTES: usize = Self::RATE_BITS / u8::BITS as usize;

    const D_SEP_A: u8 = 0x07;
    const D_SEP_B: u8 = 0x0b;
    const D_SEP_C: u8 = 0x06;

    const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
    const PAD_B: [u8; 2] = [0xff, 0xff];

    const CHAINING_VALUE_BYTE_LEN: usize = Self::CAPACITY_BITS / u8::BITS as usize;

    /// Given message (M), customization string (C) and length of C encoded using `length_encode()`
    /// function ( s.t. only first `elen` bytes are of interest ), this routine writes at max
    /// `NUM_CONTIGUOUS_BYTES_TO_READ` -bytes, starting from beginning of the `i` -th chunk
    /// ( s.t. each chunk is `CHUNK_BYTE_LEN` -bytes wide ) to the output buffer.
    /// It returns how many bytes were actually written to the output buffer.
    ///
    /// For understanding how this function works, let us assume
    ///
    /// S <- M || C || length_encode(|C|) s.t. |C| <- byte length of C
    ///
    /// We can split S into `n` -chunks s.t. first (n - 1) chunks are of length B while the
    /// last one is of length <= B. So n = ⌈|S|/ B⌉
    ///
    /// n must be 1, because it's guaranteed that S will be atleast 1 -byte wide even if both M
    /// and C are empty. Then it must be the case that 0 <= i < n.
    ///
    /// You may want to take a look at section 3.{2, 3} of the K12 specification
    /// https://keccak.team/files/KangarooTwelve.pdf for understanding why this function exists.
    #[inline(always)]
    fn get_ith_chunk<const NUM_CONTIGUOUS_BYTES_TO_READ: usize>(
        i: usize,
        msg: &[u8],
        cstr: &[u8],
        enc: &[u8],
        chunk_bytes: &mut [u8; NUM_CONTIGUOUS_BYTES_TO_READ],
    ) -> usize {
        let l0 = msg.len();
        let l1 = l0 + cstr.len();
        let l2 = l1 + enc.len();

        let mut offset = 0;
        let start_at = i * CHUNK_BYTE_LEN;

        if start_at < l0 {
            let readable = cmp::min(l0 - start_at, NUM_CONTIGUOUS_BYTES_TO_READ);
            chunk_bytes[..readable].copy_from_slice(&msg[start_at..(start_at + readable)]);

            offset += readable;
        }

        if (offset < NUM_CONTIGUOUS_BYTES_TO_READ) && ((start_at + offset) < l1) {
            let readable = cmp::min(l1 - (start_at + offset), NUM_CONTIGUOUS_BYTES_TO_READ - offset);
            let tmp = (start_at + offset) - l0;
            chunk_bytes[offset..(offset + readable)].copy_from_slice(&cstr[tmp..(tmp + readable)]);

            offset += readable;
        }

        if (offset < NUM_CONTIGUOUS_BYTES_TO_READ) && ((start_at + offset) < l2) {
            let readable = cmp::min(l2 - (start_at + offset), NUM_CONTIGUOUS_BYTES_TO_READ - offset);
            let tmp = (start_at + offset) - l1;
            chunk_bytes[offset..(offset + readable)].copy_from_slice(&enc[tmp..(tmp + readable)]);

            offset += readable;
        }

        offset
    }

    /// Given message (M) and customization string (C, which can be used for domain seperation)
    /// this routine consumes both of them into Keccak\[256\] sponge state, using a single execution thread,
    /// in chunks of `CHUNK_BYTE_LEN` -bytes s.t. returned KT128 object can be used for squeezing arbitrary number
    /// of bytes from sponge state.
    ///
    /// This is a single-threaded implementation of the KT128 tree hash mode, as described in section 3.3
    /// of the specification https://keccak.team/files/KangarooTwelve.pdf. You haven't configured this library
    /// crate to use `multi_threaded` feature.
    ///
    /// You can use this function for oneshot hashing i.e. when all the input bytes are ready to be consumed.
    #[cfg(not(feature = "multi_threaded"))]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> Self {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;

        let num_full_chunks = tlen / CHUNK_BYTE_LEN;
        let num_total_chunks = tlen.div_ceil(CHUNK_BYTE_LEN);

        let mut chunk = [0u8; CHUNK_BYTE_LEN];

        if num_total_chunks == 1 {
            let mut state = [0u64; keccak::LANE_CNT];
            let mut offset = 0;

            let clen = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen], &mut chunk);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_A }>(&mut state, &mut offset);

            Self {
                state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut cv_compressor_state = [0u64; keccak::LANE_CNT];
            let mut offset = 0;

            let mut chunk_idx = 0;

            let _ = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunk);
            chunk_idx += 1;

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &Self::PAD_A);

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                if is_x86_feature_detected!("avx2") {
                    use crate::cv::cvx4;

                    const SIMD_PARALLELISM_FACTOR: usize = 4;

                    let mut chunkx4 = [0u8; SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN];
                    let mut chaining_valuex4 = [0u8; SIMD_PARALLELISM_FACTOR * Self::CHAINING_VALUE_BYTE_LEN];

                    let simd_chunkable_till = (num_full_chunks - chunk_idx) & SIMD_PARALLELISM_FACTOR.wrapping_neg();

                    while chunk_idx < simd_chunkable_till {
                        let _ = Self::get_ith_chunk::<{ SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunkx4);
                        chunk_idx += SIMD_PARALLELISM_FACTOR;

                        unsafe {
                            cvx4::compute_chaining_valuex4::<{ Self::RATE_BITS }, { Self::D_SEP_B }, { Self::CHAINING_VALUE_BYTE_LEN }>(
                                &chunkx4,
                                &mut chaining_valuex4,
                            )
                        };

                        sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &chaining_valuex4);
                    }
                }

                if is_x86_feature_detected!("sse2") {
                    use crate::cv::cvx2;

                    const SIMD_PARALLELISM_FACTOR: usize = 2;

                    let mut chunkx2 = [0u8; SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN];
                    let mut chaining_valuex2 = [0u8; SIMD_PARALLELISM_FACTOR * Self::CHAINING_VALUE_BYTE_LEN];

                    let simd_chunkable_till = (num_full_chunks - chunk_idx) & SIMD_PARALLELISM_FACTOR.wrapping_neg();

                    while chunk_idx < simd_chunkable_till {
                        let _ = Self::get_ith_chunk::<{ SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunkx2);
                        chunk_idx += SIMD_PARALLELISM_FACTOR;

                        unsafe {
                            cvx2::compute_chaining_valuex2::<{ Self::RATE_BITS }, { Self::D_SEP_B }, { Self::CHAINING_VALUE_BYTE_LEN }>(
                                &chunkx2,
                                &mut chaining_valuex2,
                            )
                        };

                        sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &chaining_valuex2);
                    }
                }
            }

            while chunk_idx < num_total_chunks {
                let clen = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunk);
                chunk_idx += 1;

                let mut cv = [0u8; Self::CHAINING_VALUE_BYTE_LEN];

                let mut hasher = TurboShake128::default();
                let _ = hasher.absorb(&chunk[..clen]);
                let _ = hasher.finalize::<{ Self::D_SEP_B }>();
                let _ = hasher.squeeze(&mut cv);

                sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &cv);
            }

            let (enc, elen) = length_encode(num_total_chunks - 1);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &enc[..elen]);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &Self::PAD_B);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_C }>(&mut cv_compressor_state, &mut offset);

            Self {
                state: cv_compressor_state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        }
    }

    /// Given message (M) and customization string (C, which can be used for domain seperation)
    /// this routine consumes both of them into Keccak\[256\] sponge state, using multiple threads i.e.
    /// equals to # -of logical cores supported by execution environment, in chunks of `CHUNK_BYTE_LEN` -bytes s.t.
    /// returned KT128 object can be used for squeezing arbitrary number of bytes from sponge state.
    ///
    /// This is a multi-threaded implementation of the KT128 tree hash mode, as described in section 3.3
    /// of the specification https://keccak.team/files/KangarooTwelve.pdf. You're using this function because
    /// you have configured this library crate to use `multi_threaded` feature.
    ///
    /// You can use this function for oneshot hashing i.e. when all the input bytes are ready to be consumed.
    #[cfg(feature = "multi_threaded")]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> Self {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;

        let num_full_chunks = tlen / CHUNK_BYTE_LEN;
        let num_total_chunks = tlen.div_ceil(CHUNK_BYTE_LEN);

        let mut chunk = [0u8; CHUNK_BYTE_LEN];

        if num_total_chunks == 1 {
            let mut state = [0u64; keccak::LANE_CNT];
            let mut offset = 0;

            let clen = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen], &mut chunk);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_A }>(&mut state, &mut offset);

            Self {
                state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut cv_compressor_state = [0u64; keccak::LANE_CNT];
            let mut offset = 0;

            let mut chunk_idx = 0;

            let _ = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(0, msg, cstr, &enc[..elen], &mut chunk);
            chunk_idx += 1;

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &Self::PAD_A);

            let simd_parallelism_factor: usize = {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    if is_x86_feature_detected!("avx2") {
                        4
                    } else if is_x86_feature_detected!("sse2") {
                        2
                    } else {
                        1
                    }
                }

                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                {
                    1
                }
            };

            let num_simd_compressions = (num_full_chunks - chunk_idx) / simd_parallelism_factor;
            if num_simd_compressions > num_cpus::get() {
                let pool = ThreadPoolBuilder::new().num_threads(num_cpus::get()).build().unwrap();

                let cvs = pool.install(|| {
                    let num_simd_compressions_per_cpu = num_simd_compressions.div_ceil(num_cpus::get());

                    let num_cv_bytes_per_simd_compression = simd_parallelism_factor * Self::CHAINING_VALUE_BYTE_LEN;
                    let num_cv_bytes_per_cpu = num_simd_compressions_per_cpu * num_cv_bytes_per_simd_compression;

                    let total_num_cv_bytes = num_simd_compressions * num_cv_bytes_per_simd_compression;
                    let mut cvs = vec![0u8; total_num_cv_bytes];

                    cvs.par_chunks_mut(num_cv_bytes_per_cpu).enumerate().for_each(|(cpu_idx, chaining_values)| {
                        let simd_compression_idx_starts_at = cpu_idx * num_simd_compressions_per_cpu;
                        let simd_compression_idx_ends_at = (simd_compression_idx_starts_at + num_simd_compressions_per_cpu).min(num_simd_compressions);

                        let num_local_simd_compressions = simd_compression_idx_ends_at - simd_compression_idx_starts_at;
                        let num_local_chunks = num_local_simd_compressions * simd_parallelism_factor;

                        let local_chunk_idx_starts_at = chunk_idx + (simd_compression_idx_starts_at * simd_parallelism_factor);
                        let local_chunk_idx_ends_at = local_chunk_idx_starts_at + num_local_chunks;

                        let mut local_chunk_idx = local_chunk_idx_starts_at;

                        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                        {
                            if is_x86_feature_detected!("avx2") {
                                use crate::cv::cvx4;

                                const SIMD_PARALLELISM_FACTOR: usize = 4;
                                let mut chunkx4 = [0u8; SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN];

                                let mut cv_bytes_offset = 0;
                                while local_chunk_idx < local_chunk_idx_ends_at {
                                    let _ = Self::get_ith_chunk::<{ SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN }>(
                                        local_chunk_idx,
                                        msg,
                                        cstr,
                                        &enc[..elen],
                                        &mut chunkx4,
                                    );
                                    local_chunk_idx += simd_parallelism_factor;

                                    unsafe {
                                        cvx4::compute_chaining_valuex4::<{ Self::RATE_BITS }, { Self::D_SEP_B }, { Self::CHAINING_VALUE_BYTE_LEN }>(
                                            &chunkx4,
                                            &mut chaining_values[cv_bytes_offset..(cv_bytes_offset + num_cv_bytes_per_simd_compression)],
                                        )
                                    };

                                    cv_bytes_offset += num_cv_bytes_per_simd_compression;
                                }

                                return;
                            }

                            if is_x86_feature_detected!("sse2") {
                                use crate::cv::cvx2;

                                const SIMD_PARALLELISM_FACTOR: usize = 2;
                                let mut chunkx2 = [0u8; SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN];

                                let mut cv_bytes_offset = 0;
                                while local_chunk_idx < local_chunk_idx_ends_at {
                                    let _ = Self::get_ith_chunk::<{ SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN }>(
                                        local_chunk_idx,
                                        msg,
                                        cstr,
                                        &enc[..elen],
                                        &mut chunkx2,
                                    );
                                    local_chunk_idx += simd_parallelism_factor;

                                    unsafe {
                                        cvx2::compute_chaining_valuex2::<{ Self::RATE_BITS }, { Self::D_SEP_B }, { Self::CHAINING_VALUE_BYTE_LEN }>(
                                            &chunkx2,
                                            &mut chaining_values[cv_bytes_offset..(cv_bytes_offset + num_cv_bytes_per_simd_compression)],
                                        )
                                    };

                                    cv_bytes_offset += num_cv_bytes_per_simd_compression;
                                }

                                return;
                            }
                        }

                        let mut chunk = [0u8; CHUNK_BYTE_LEN];

                        let mut cv_bytes_offset = 0;
                        while local_chunk_idx < local_chunk_idx_ends_at {
                            let _ = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(local_chunk_idx, msg, cstr, &enc[..elen], &mut chunk);
                            local_chunk_idx += 1;

                            let mut hasher = TurboShake128::default();
                            let _ = hasher.absorb(&chunk);
                            let _ = hasher.finalize::<{ Self::D_SEP_B }>();
                            let _ = hasher.squeeze(&mut chaining_values[cv_bytes_offset..(cv_bytes_offset + Self::CHAINING_VALUE_BYTE_LEN)]);

                            cv_bytes_offset += Self::CHAINING_VALUE_BYTE_LEN;
                        }
                    });

                    chunk_idx += num_simd_compressions * simd_parallelism_factor;
                    cvs
                });

                sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &cvs);
            }

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                if is_x86_feature_detected!("avx2") {
                    use crate::cv::cvx4;

                    const SIMD_PARALLELISM_FACTOR: usize = 4;

                    let mut chunkx4 = [0u8; SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN];
                    let mut chaining_valuex4 = [0u8; SIMD_PARALLELISM_FACTOR * Self::CHAINING_VALUE_BYTE_LEN];

                    let simd_chunkable_till = (num_full_chunks - chunk_idx) & SIMD_PARALLELISM_FACTOR.wrapping_neg();

                    while chunk_idx < simd_chunkable_till {
                        let _ = Self::get_ith_chunk::<{ SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunkx4);
                        chunk_idx += SIMD_PARALLELISM_FACTOR;

                        unsafe {
                            cvx4::compute_chaining_valuex4::<{ Self::RATE_BITS }, { Self::D_SEP_B }, { Self::CHAINING_VALUE_BYTE_LEN }>(
                                &chunkx4,
                                &mut chaining_valuex4,
                            )
                        };

                        sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &chaining_valuex4);
                    }
                }

                if is_x86_feature_detected!("sse2") {
                    use crate::cv::cvx2;

                    const SIMD_PARALLELISM_FACTOR: usize = 2;

                    let mut chunkx2 = [0u8; SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN];
                    let mut chaining_valuex2 = [0u8; SIMD_PARALLELISM_FACTOR * Self::CHAINING_VALUE_BYTE_LEN];

                    let simd_chunkable_till = (num_full_chunks - chunk_idx) & SIMD_PARALLELISM_FACTOR.wrapping_neg();

                    while chunk_idx < simd_chunkable_till {
                        let _ = Self::get_ith_chunk::<{ SIMD_PARALLELISM_FACTOR * CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunkx2);
                        chunk_idx += SIMD_PARALLELISM_FACTOR;

                        unsafe {
                            cvx2::compute_chaining_valuex2::<{ Self::RATE_BITS }, { Self::D_SEP_B }, { Self::CHAINING_VALUE_BYTE_LEN }>(
                                &chunkx2,
                                &mut chaining_valuex2,
                            )
                        };

                        sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &chaining_valuex2);
                    }
                }
            }

            while chunk_idx < num_total_chunks {
                let clen = Self::get_ith_chunk::<{ CHUNK_BYTE_LEN }>(chunk_idx, msg, cstr, &enc[..elen], &mut chunk);
                chunk_idx += 1;

                let mut cv = [0u8; Self::CHAINING_VALUE_BYTE_LEN];

                let mut hasher = TurboShake128::default();
                let _ = hasher.absorb(&chunk[..clen]);
                let _ = hasher.finalize::<{ Self::D_SEP_B }>();
                let _ = hasher.squeeze(&mut cv);

                sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &cv);
            }

            let (enc, elen) = length_encode(num_total_chunks - 1);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &enc[..elen]);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut cv_compressor_state, &mut offset, &Self::PAD_B);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_C }>(&mut cv_compressor_state, &mut offset);

            Self {
                state: cv_compressor_state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        }
    }

    /// Given that N -bytes input message ( along with customization string ) is already
    /// absorbed into sponge state, this routine is used for squeezing M -bytes out of
    /// consumable part of the sponge state ( i.e. rate portion of the state ).
    ///
    /// Note, this routine can be called arbitrary number of times, for squeezing arbitrary
    /// number of bytes from sponge Keccak\[256\].
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        if self.is_ready != usize::MAX {
            return;
        }

        sponge::squeeze::<{ Self::RATE_BYTES }>(&mut self.state, &mut self.squeezable, out);
    }
}
