use crate::utils::{get_ith_chunk, length_encode};
use turboshake::{TurboShake256, sponge};

#[cfg(feature = "multi_threaded")]
use std::cmp;

#[cfg(feature = "multi_threaded")]
use rayon::{ThreadPoolBuilder, prelude::*};

/// KT256 Extendable Output Function (XOF)
///
/// See <https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve>
pub struct KT256;

#[derive(Copy, Clone)]
pub struct KT256XOF {
    state: [u64; turboshake::keccak::LANE_CNT],
    squeezable: usize,
}

impl KT256 {
    const CAPACITY_BITS: usize = 512;
    const KECCAK_STATE_BIT_WIDTH: usize = turboshake::keccak::LANE_CNT * turboshake::keccak::W;
    const RATE_BITS: usize = Self::KECCAK_STATE_BIT_WIDTH - Self::CAPACITY_BITS;
    const RATE_BYTES: usize = Self::RATE_BITS / u8::BITS as usize;
    const B: usize = 8192;
    const D_SEP_A: u8 = 0x07;
    const D_SEP_B: u8 = 0x0b;
    const D_SEP_C: u8 = 0x06;

    /// Given message (M) and customization string (C, which can be used for domain seperation)
    /// this routine consumes both of them into Keccak\[512\] sponge state, using single thread,
    /// in chunks of B -bytes s.t. returned KT256 object can be used for squeezing arbitrary number
    /// of bytes from sponge state.
    ///
    /// This is a single-threaded implementation of the KT256 tree hash mode, as described in section 3.4
    /// of the specification https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve. You haven't
    /// configured this library crate to use `multi_threaded` feature.
    ///
    /// You can use this function for oneshot hashing i.e. when all the input bytes are ready to be consumed.
    #[cfg(not(any(feature = "cuda", feature = "multi_threaded")))]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> KT256XOF {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = tlen.div_ceil(Self::B);

        if n == 1 {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, clen) = get_ith_chunk::<{ Self::B }>(0, msg, cstr, &enc[..elen]);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_A }>(&mut state, &mut offset);

            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, _) = get_ith_chunk::<{ Self::B }>(0, msg, cstr, &enc[..elen]);
            const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
            const PAD_B: [u8; 2] = [0xff, 0xff];

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &PAD_A);

            for i in 1..n {
                let (chunk, clen) = get_ith_chunk::<{ Self::B }>(i, msg, cstr, &enc[..elen]);
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

    /// Given message (M) and customization string (C, which can be used for domain seperation)
    /// this routine consumes both of them into Keccak\[512\] sponge state, using multiple threads i.e.
    /// equals to # -of logical cores supported by execution environment, in chunks of B -bytes s.t.
    /// returned KT256 object can be used for squeezing arbitrary number of bytes from sponge state.
    ///
    /// This is a multi-threaded implementation of the KT256 tree hash mode, as described in section 3.4
    /// of the specification https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve. You're using
    /// this function because you have configured this library crate to use `multi_threaded` feature.
    ///
    /// You can use this function for oneshot hashing i.e. when all the input bytes are ready to be consumed.
    #[cfg(feature = "multi_threaded")]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> KT256XOF {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = tlen.div_ceil(Self::B);

        if n == 1 {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, clen) = get_ith_chunk::<{ Self::B }>(0, msg, cstr, &enc[..elen]);

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::D_SEP_A }>(&mut state, &mut offset);

            KT256XOF {
                state,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; turboshake::keccak::LANE_CNT];
            let mut offset = 0;

            let (chunk, _) = get_ith_chunk::<{ Self::B }>(0, msg, cstr, &enc[..elen]);
            const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
            const PAD_B: [u8; 2] = [0xff, 0xff];

            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }>(&mut state, &mut offset, &PAD_A);

            let cpus = cmp::min(num_cpus::get(), n - 1);
            let pool = ThreadPoolBuilder::new().num_threads(cpus).build().unwrap();
            let cvs = pool.install(|| {
                let mut cvs = vec![0u8; (n - 1) * 64];

                cvs.par_chunks_mut(64).enumerate().for_each(|(i, cv)| {
                    let (chunk, clen) = get_ith_chunk::<{ Self::B }>(i + 1, msg, cstr, &enc[..elen]);

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

    #[cfg(feature = "cuda")]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> KT256XOF {
        let state = crate::cuda::kt256_absorb_state(msg, cstr).unwrap_or_else(|e| panic!("KT256 GPU hashing failed: {e}"));

        KT256XOF {
            state,
            squeezable: Self::RATE_BYTES,
        }
    }
}

impl KT256XOF {
    /// Given that N -bytes input message ( along with customization string ) is already
    /// absorbed into sponge state, this routine is used for squeezing M -bytes out of
    /// consumable part of the sponge state ( i.e. rate portion of the state ).
    ///
    /// Note, this routine can be called arbitrary number of times, for squeezing arbitrary
    /// number of bytes from sponge Keccak\[512\].
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        sponge::squeeze::<{ KT256::RATE_BYTES }>(&mut self.state, &mut self.squeezable, out);
    }
}
