use crate::utils::length_encode;
use std::cmp;
use turboshake::{TurboShake256, sponge};

#[cfg(feature = "multi_threaded")]
use rayon::{ThreadPoolBuilder, prelude::*};

/// KT256 Extendable Output Function (XOF)
///
/// See <https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve>
#[derive(Copy, Clone)]
pub struct KT256 {
    state: [u64; 25],
    is_ready: usize,
    squeezable: usize,
}

impl KT256 {
    const CAPACITY_BITS: usize = 512;
    const RATE_BITS: usize = 1600 - Self::CAPACITY_BITS;
    const RATE_BYTES: usize = Self::RATE_BITS / 8;
    const RATE_WORDS: usize = Self::RATE_BYTES / 8;
    const B: usize = 8192;
    const D_SEP_A: u8 = 0x07;
    const D_SEP_B: u8 = 0x0b;
    const D_SEP_C: u8 = 0x06;

    /// Given message (M), customization string (C) and length of C encoded using `length_encode()`
    /// function ( s.t. only first `elen` bytes are of interest ), this routine extracts out `i` -th
    /// chunk ( s.t. each chunk is `B` -bytes wide ) along with how many ( must be <= B ) bytes
    /// of that chunk are of significance.
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
    fn get_ith_chunk(i: usize, msg: &[u8], cstr: &[u8], enc: &[u8]) -> ([u8; Self::B], usize) {
        let l0 = msg.len();
        let l1 = l0 + cstr.len();
        let l2 = l1 + enc.len();

        let mut res = [0u8; Self::B];
        let mut off = 0;

        let start_at = i * Self::B;

        if start_at < l0 {
            let readable = cmp::min(l0 - start_at, Self::B);
            res[..readable].copy_from_slice(&msg[start_at..(start_at + readable)]);

            off += readable;
        }

        if (off < Self::B) && ((start_at + off) < l1) {
            let readable = cmp::min(l1 - (start_at + off), Self::B - off);
            let tmp = (start_at + off) - l0;
            res[off..(off + readable)].copy_from_slice(&cstr[tmp..(tmp + readable)]);

            off += readable;
        }

        if (off < Self::B) && ((start_at + off) < l2) {
            let readable = cmp::min(l2 - (start_at + off), Self::B - off);
            let tmp = (start_at + off) - l1;
            res[off..(off + readable)].copy_from_slice(&enc[tmp..(tmp + readable)]);

            off += readable;
        }

        (res, off)
    }

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
    #[cfg(not(feature = "multi_threaded"))]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> Self {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = tlen.div_ceil(Self::B);

        if n == 1 {
            let mut state = [0u64; 25];
            let mut offset = 0;

            let (chunk, clen) = Self::get_ith_chunk(0, msg, cstr, &enc[..elen]);

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }, { Self::D_SEP_A }>(&mut state, &mut offset);

            Self {
                state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; 25];
            let mut offset = 0;

            let (chunk, _) = Self::get_ith_chunk(0, msg, cstr, &enc[..elen]);
            const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
            const PAD_B: [u8; 2] = [0xff, 0xff];

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &PAD_A);

            for i in 1..n {
                let (chunk, clen) = Self::get_ith_chunk(i, msg, cstr, &enc[..elen]);
                let mut cv = [0u8; 64];

                let mut hasher = TurboShake256::new();
                hasher.absorb(&chunk[..clen]);
                hasher.finalize::<{ Self::D_SEP_B }>();
                hasher.squeeze(&mut cv);

                sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &cv);
            }

            let (enc, elen) = length_encode(n - 1);

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &enc[..elen]);
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &PAD_B);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }, { Self::D_SEP_C }>(&mut state, &mut offset);

            Self {
                state,
                is_ready: usize::MAX,
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
    pub fn hash(msg: &[u8], cstr: &[u8]) -> Self {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = (tlen + (Self::B - 1)) / Self::B;

        if n == 1 {
            let mut state = [0u64; 25];
            let mut offset = 0;

            let (chunk, clen) = Self::get_ith_chunk(0, msg, cstr, &enc[..elen]);

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &chunk[..clen]);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }, { Self::D_SEP_A }>(&mut state, &mut offset);

            Self {
                state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; 25];
            let mut offset = 0;

            let (chunk, _) = Self::get_ith_chunk(0, msg, cstr, &enc[..elen]);
            const PAD_A: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
            const PAD_B: [u8; 2] = [0xff, 0xff];

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &chunk);
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &PAD_A);

            let cpus = cmp::min(num_cpus::get(), n - 1);
            let pool = ThreadPoolBuilder::new().num_threads(cpus).build().unwrap();
            let cvs = pool.install(|| {
                let mut cvs = vec![0u8; (n - 1) * 64];

                cvs.par_chunks_mut(64).enumerate().for_each(|(i, cv)| {
                    let (chunk, clen) = Self::get_ith_chunk(i + 1, msg, cstr, &enc[..elen]);

                    let mut hasher = TurboShake256::new();
                    hasher.absorb(&chunk[..clen]);
                    hasher.finalize::<{ Self::D_SEP_B }>();
                    hasher.squeeze(cv);
                });

                cvs
            });

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &cvs);

            let (enc, elen) = length_encode(n - 1);

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &enc[..elen]);
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut state, &mut offset, &PAD_B);
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }, { Self::D_SEP_C }>(&mut state, &mut offset);

            Self {
                state,
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
    /// number of bytes from sponge Keccak\[512\].
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        if self.is_ready != usize::MAX {
            return;
        }

        sponge::squeeze::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(&mut self.state, &mut self.squeezable, out);
    }
}
