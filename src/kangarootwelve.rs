use crate::utils::length_encode;
use std::cmp;
use turboshake::sponge;

#[derive(Copy, Clone)]
struct KangarooTwelve {
    state: [u64; 25],
    is_ready: usize,
    squeezable: usize,
}

impl KangarooTwelve {
    const CAPACITY_BITS: usize = 256;
    const RATE_BITS: usize = 1600 - Self::CAPACITY_BITS;
    const RATE_BYTES: usize = Self::RATE_BITS / 8;
    const RATE_WORDS: usize = Self::RATE_BYTES / 8;
    const B: usize = 8192;
    const D_SEP_A: u8 = 0x07;
    const D_SEP_B: u8 = 0x0b;
    const D_SEP_C: u8 = 0x06;

    /// Create a new instance of K12 Extendable Output Function (XOF), into which
    /// arbitrary number of message bytes can be absorbed and arbitrary many bytes
    /// can be squeezed out.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            state: [0u64; 25],
            is_ready: usize::MIN,
            squeezable: 0,
        }
    }

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
    #[inline(always)]
    fn get_ith_chunk(
        i: usize,
        msg: &[u8],
        cstr: &[u8],
        enc: &[u8],
        elen: usize,
    ) -> ([u8; Self::B], usize) {
        let l0 = msg.len();
        let l1 = l0 + cstr.len();
        let l2 = l1 + elen;

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

    #[inline(always)]
    pub fn hash(msg: &[u8], cstr: &[u8]) -> Self {
        let (enc, elen) = length_encode(cstr.len());
        let tlen = msg.len() + cstr.len() + elen;
        let n = (tlen + (Self::B - 1)) / Self::B;

        if n == 1 {
            let mut state = [0u64; 25];
            let mut offset = 0;

            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
                &mut state,
                &mut offset,
                msg,
            );
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
                &mut state,
                &mut offset,
                cstr,
            );
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
                &mut state,
                &mut offset,
                &enc[..elen],
            );
            sponge::finalize::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }, { Self::D_SEP_A }>(
                &mut state,
                &mut offset,
            );

            Self {
                state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        } else {
            let mut state = [0u64; 25];
            let mut offset = 0;

            let mut moff = 0;
            let mut coff = 0;
            let mut eoff = 0;
            let mut consumed = 0;

            let mut consumed = 0;
            let readable = cmp::min(msg.len(), Self::B);
            sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
                &mut state,
                &mut offset,
                &msg[..readable],
            );

            consumed += readable;
            if consumed < Self::B {
                let readable = cmp::min(cstr.len(), Self::B - consumed);
                sponge::absorb::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
                    &mut state,
                    &mut offset,
                    &cstr[..readable],
                );

                consumed += readable;
            }

            Self {
                state,
                is_ready: usize::MAX,
                squeezable: Self::RATE_BYTES,
            }
        }
    }

    /// Given that N -bytes input message is already absorbed into sponge state, this
    /// routine is used for squeezing M -bytes out of consumable part of sponge state
    /// ( i.e. rate portion of the state )
    ///
    /// Note, this routine can be called arbitrary number of times, for squeezing arbitrary
    /// number of bytes from sponge Keccak\[256\].
    ///
    /// Make sure you absorb message bytes first, then only call this function, otherwise
    /// it can't squeeze anything out.
    ///
    /// Adapted from https://github.com/itzmeanjan/turboshake/blob/81243e8ebe792b8af53abf6b8a9dae6744949896/src/turboshake128.rs#L87-L109
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        if self.is_ready != usize::MAX {
            return;
        }

        sponge::squeeze::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
            &mut self.state,
            &mut self.squeezable,
            out,
        );
    }
}
