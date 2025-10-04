use crate::keccak::consts::{RC, ROT};
use turboshake::keccak;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Keccak-p\[1600\] permutation, applying 12 rounds permutation, on four states, each of dimension 5 x 5 x 64 ( = 1600 -bits ),
/// using AVX512, following <https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L527-L540>.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
#[allow(unused_unsafe)]
pub unsafe fn permute(state: &mut [__m512i; keccak::LANE_CNT]) {
    unsafe {
        roundx4(state, 0);
        roundx4(state, 4);
        roundx4(state, 8);
    }
}

/// Keccak-p\[1600\] round function, applying all five step mapping functions in order, for four consecutive rounds, starting from round index `ridx`.
/// This function is a line by line translation of <https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L114-L525>
/// to AVX512 compatible version, where we are permuting four Keccak-p\[1600\] permutation instances in parallel.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
#[allow(unused_unsafe)]
#[inline]
unsafe fn roundx4(state: &mut [__m512i; keccak::LANE_CNT], ridx: usize) {
    unsafe {
        let mut c = [_mm512_setzero_si512(); 5];
        let mut d = [_mm512_setzero_si512(); 5];
        let mut t: __m512i;

        // Round ridx + 0
        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm512_xor_si512(c[0], state[i]);
            c[1] = _mm512_xor_si512(c[1], state[i + 1]);
            c[2] = _mm512_xor_si512(c[2], state[i + 2]);
            c[3] = _mm512_xor_si512(c[3], state[i + 3]);
            c[4] = _mm512_xor_si512(c[4], state[i + 4]);
        }

        d[0] = _mm512_xor_si512(c[4], _mm512_or_si512(_mm512_slli_epi64(c[1], 1), _mm512_srli_epi64(c[1], keccak::W as u32 - 1)));
        d[1] = _mm512_xor_si512(c[0], _mm512_or_si512(_mm512_slli_epi64(c[2], 1), _mm512_srli_epi64(c[2], keccak::W as u32 - 1)));
        d[2] = _mm512_xor_si512(c[1], _mm512_or_si512(_mm512_slli_epi64(c[3], 1), _mm512_srli_epi64(c[3], keccak::W as u32 - 1)));
        d[3] = _mm512_xor_si512(c[2], _mm512_or_si512(_mm512_slli_epi64(c[4], 1), _mm512_srli_epi64(c[4], keccak::W as u32 - 1)));
        d[4] = _mm512_xor_si512(c[3], _mm512_or_si512(_mm512_slli_epi64(c[0], 1), _mm512_srli_epi64(c[0], keccak::W as u32 - 1)));

        c[0] = _mm512_xor_si512(state[0], d[0]);
        t = _mm512_xor_si512(state[6], d[1]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[6] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[6] as u32));
        t = _mm512_xor_si512(state[12], d[2]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[12] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[12] as u32));
        t = _mm512_xor_si512(state[18], d[3]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[18] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[18] as u32));
        t = _mm512_xor_si512(state[24], d[4]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[24] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[24] as u32));

        state[0] = _mm512_xor_si512(_mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2])), _mm512_set1_epi64(RC[ridx]));
        state[6] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[12] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[18] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[24] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[10], d[0]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[10] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[10] as u32));
        t = _mm512_xor_si512(state[16], d[1]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[16] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[16] as u32));
        t = _mm512_xor_si512(state[22], d[2]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[22] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[22] as u32));
        t = _mm512_xor_si512(state[3], d[3]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[3] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[3] as u32));
        t = _mm512_xor_si512(state[9], d[4]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[9] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[9] as u32));

        state[10] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[16] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[22] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[3] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[9] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[20], d[0]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[20] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[20] as u32));
        t = _mm512_xor_si512(state[1], d[1]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[1] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[1] as u32));
        t = _mm512_xor_si512(state[7], d[2]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[7] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[7] as u32));
        t = _mm512_xor_si512(state[13], d[3]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[13] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[13] as u32));
        t = _mm512_xor_si512(state[19], d[4]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[19] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[19] as u32));

        state[20] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[1] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[7] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[13] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[19] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[5], d[0]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[5] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[5] as u32));
        t = _mm512_xor_si512(state[11], d[1]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[11] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[11] as u32));
        t = _mm512_xor_si512(state[17], d[2]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[17] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[17] as u32));
        t = _mm512_xor_si512(state[23], d[3]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[23] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[23] as u32));
        t = _mm512_xor_si512(state[4], d[4]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[4] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[4] as u32));

        state[5] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[11] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[17] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[23] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[4] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[15], d[0]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[15] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[15] as u32));
        t = _mm512_xor_si512(state[21], d[1]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[21] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[21] as u32));
        t = _mm512_xor_si512(state[2], d[2]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[2] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[2] as u32));
        t = _mm512_xor_si512(state[8], d[3]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[8] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[8] as u32));
        t = _mm512_xor_si512(state[14], d[4]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[14] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[14] as u32));

        state[15] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[21] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[2] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[8] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[14] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        // Round ridx + 1
        c.fill(_mm512_setzero_si512());

        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm512_xor_si512(c[0], state[i]);
            c[1] = _mm512_xor_si512(c[1], state[i + 1]);
            c[2] = _mm512_xor_si512(c[2], state[i + 2]);
            c[3] = _mm512_xor_si512(c[3], state[i + 3]);
            c[4] = _mm512_xor_si512(c[4], state[i + 4]);
        }

        d[0] = _mm512_xor_si512(c[4], _mm512_or_si512(_mm512_slli_epi64(c[1], 1), _mm512_srli_epi64(c[1], keccak::W as u32 - 1)));
        d[1] = _mm512_xor_si512(c[0], _mm512_or_si512(_mm512_slli_epi64(c[2], 1), _mm512_srli_epi64(c[2], keccak::W as u32 - 1)));
        d[2] = _mm512_xor_si512(c[1], _mm512_or_si512(_mm512_slli_epi64(c[3], 1), _mm512_srli_epi64(c[3], keccak::W as u32 - 1)));
        d[3] = _mm512_xor_si512(c[2], _mm512_or_si512(_mm512_slli_epi64(c[4], 1), _mm512_srli_epi64(c[4], keccak::W as u32 - 1)));
        d[4] = _mm512_xor_si512(c[3], _mm512_or_si512(_mm512_slli_epi64(c[0], 1), _mm512_srli_epi64(c[0], keccak::W as u32 - 1)));

        c[0] = _mm512_xor_si512(state[0], d[0]);
        t = _mm512_xor_si512(state[16], d[1]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[6] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[6] as u32));
        t = _mm512_xor_si512(state[7], d[2]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[12] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[12] as u32));
        t = _mm512_xor_si512(state[23], d[3]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[18] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[18] as u32));
        t = _mm512_xor_si512(state[14], d[4]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[24] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[24] as u32));

        state[0] = _mm512_xor_si512(_mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2])), _mm512_set1_epi64(RC[ridx + 1]));
        state[16] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[7] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[23] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[14] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[20], d[0]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[10] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[10] as u32));
        t = _mm512_xor_si512(state[11], d[1]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[16] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[16] as u32));
        t = _mm512_xor_si512(state[2], d[2]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[22] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[22] as u32));
        t = _mm512_xor_si512(state[18], d[3]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[3] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[3] as u32));
        t = _mm512_xor_si512(state[9], d[4]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[9] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[9] as u32));

        state[20] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[11] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[2] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[18] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[9] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[15], d[0]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[20] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[20] as u32));
        t = _mm512_xor_si512(state[6], d[1]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[1] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[1] as u32));
        t = _mm512_xor_si512(state[22], d[2]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[7] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[7] as u32));
        t = _mm512_xor_si512(state[13], d[3]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[13] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[13] as u32));
        t = _mm512_xor_si512(state[4], d[4]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[19] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[19] as u32));

        state[15] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[6] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[22] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[13] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[4] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[10], d[0]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[5] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[5] as u32));
        t = _mm512_xor_si512(state[1], d[1]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[11] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[11] as u32));
        t = _mm512_xor_si512(state[17], d[2]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[17] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[17] as u32));
        t = _mm512_xor_si512(state[8], d[3]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[23] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[23] as u32));
        t = _mm512_xor_si512(state[24], d[4]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[4] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[4] as u32));

        state[10] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[1] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[17] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[8] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[24] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[5], d[0]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[15] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[15] as u32));
        t = _mm512_xor_si512(state[21], d[1]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[21] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[21] as u32));
        t = _mm512_xor_si512(state[12], d[2]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[2] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[2] as u32));
        t = _mm512_xor_si512(state[3], d[3]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[8] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[8] as u32));
        t = _mm512_xor_si512(state[19], d[4]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[14] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[14] as u32));

        state[5] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[21] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[12] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[3] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[19] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        // Round ridx + 2
        c.fill(_mm512_setzero_si512());

        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm512_xor_si512(c[0], state[i]);
            c[1] = _mm512_xor_si512(c[1], state[i + 1]);
            c[2] = _mm512_xor_si512(c[2], state[i + 2]);
            c[3] = _mm512_xor_si512(c[3], state[i + 3]);
            c[4] = _mm512_xor_si512(c[4], state[i + 4]);
        }

        d[0] = _mm512_xor_si512(c[4], _mm512_or_si512(_mm512_slli_epi64(c[1], 1), _mm512_srli_epi64(c[1], keccak::W as u32 - 1)));
        d[1] = _mm512_xor_si512(c[0], _mm512_or_si512(_mm512_slli_epi64(c[2], 1), _mm512_srli_epi64(c[2], keccak::W as u32 - 1)));
        d[2] = _mm512_xor_si512(c[1], _mm512_or_si512(_mm512_slli_epi64(c[3], 1), _mm512_srli_epi64(c[3], keccak::W as u32 - 1)));
        d[3] = _mm512_xor_si512(c[2], _mm512_or_si512(_mm512_slli_epi64(c[4], 1), _mm512_srli_epi64(c[4], keccak::W as u32 - 1)));
        d[4] = _mm512_xor_si512(c[3], _mm512_or_si512(_mm512_slli_epi64(c[0], 1), _mm512_srli_epi64(c[0], keccak::W as u32 - 1)));

        c[0] = _mm512_xor_si512(state[0], d[0]);
        t = _mm512_xor_si512(state[11], d[1]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[6] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[6] as u32));
        t = _mm512_xor_si512(state[22], d[2]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[12] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[12] as u32));
        t = _mm512_xor_si512(state[8], d[3]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[18] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[18] as u32));
        t = _mm512_xor_si512(state[19], d[4]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[24] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[24] as u32));

        state[0] = _mm512_xor_si512(_mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2])), _mm512_set1_epi64(RC[ridx + 2]));
        state[11] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[22] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[8] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[19] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[15], d[0]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[10] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[10] as u32));
        t = _mm512_xor_si512(state[1], d[1]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[16] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[16] as u32));
        t = _mm512_xor_si512(state[12], d[2]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[22] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[22] as u32));
        t = _mm512_xor_si512(state[23], d[3]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[3] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[3] as u32));
        t = _mm512_xor_si512(state[9], d[4]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[9] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[9] as u32));

        state[15] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[1] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[12] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[23] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[9] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[5], d[0]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[20] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[20] as u32));
        t = _mm512_xor_si512(state[16], d[1]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[1] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[1] as u32));
        t = _mm512_xor_si512(state[2], d[2]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[7] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[7] as u32));
        t = _mm512_xor_si512(state[13], d[3]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[13] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[13] as u32));
        t = _mm512_xor_si512(state[24], d[4]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[19] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[19] as u32));

        state[5] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[16] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[2] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[13] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[24] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[20], d[0]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[5] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[5] as u32));
        t = _mm512_xor_si512(state[6], d[1]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[11] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[11] as u32));
        t = _mm512_xor_si512(state[17], d[2]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[17] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[17] as u32));
        t = _mm512_xor_si512(state[3], d[3]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[23] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[23] as u32));
        t = _mm512_xor_si512(state[14], d[4]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[4] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[4] as u32));

        state[20] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[6] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[17] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[3] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[14] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[10], d[0]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[15] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[15] as u32));
        t = _mm512_xor_si512(state[21], d[1]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[21] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[21] as u32));
        t = _mm512_xor_si512(state[7], d[2]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[2] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[2] as u32));
        t = _mm512_xor_si512(state[18], d[3]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[8] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[8] as u32));
        t = _mm512_xor_si512(state[4], d[4]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[14] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[14] as u32));

        state[10] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[21] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[7] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[18] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[4] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        // Round ridx + 3
        c.fill(_mm512_setzero_si512());

        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm512_xor_si512(c[0], state[i]);
            c[1] = _mm512_xor_si512(c[1], state[i + 1]);
            c[2] = _mm512_xor_si512(c[2], state[i + 2]);
            c[3] = _mm512_xor_si512(c[3], state[i + 3]);
            c[4] = _mm512_xor_si512(c[4], state[i + 4]);
        }

        d[0] = _mm512_xor_si512(c[4], _mm512_or_si512(_mm512_slli_epi64(c[1], 1), _mm512_srli_epi64(c[1], keccak::W as u32 - 1)));
        d[1] = _mm512_xor_si512(c[0], _mm512_or_si512(_mm512_slli_epi64(c[2], 1), _mm512_srli_epi64(c[2], keccak::W as u32 - 1)));
        d[2] = _mm512_xor_si512(c[1], _mm512_or_si512(_mm512_slli_epi64(c[3], 1), _mm512_srli_epi64(c[3], keccak::W as u32 - 1)));
        d[3] = _mm512_xor_si512(c[2], _mm512_or_si512(_mm512_slli_epi64(c[4], 1), _mm512_srli_epi64(c[4], keccak::W as u32 - 1)));
        d[4] = _mm512_xor_si512(c[3], _mm512_or_si512(_mm512_slli_epi64(c[0], 1), _mm512_srli_epi64(c[0], keccak::W as u32 - 1)));

        c[0] = _mm512_xor_si512(state[0], d[0]);
        t = _mm512_xor_si512(state[1], d[1]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[6] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[6] as u32));
        t = _mm512_xor_si512(state[2], d[2]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[12] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[12] as u32));
        t = _mm512_xor_si512(state[3], d[3]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[18] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[18] as u32));
        t = _mm512_xor_si512(state[4], d[4]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[24] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[24] as u32));

        state[0] = _mm512_xor_si512(_mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2])), _mm512_set1_epi64(RC[ridx + 3]));
        state[1] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[2] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[3] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[4] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[5], d[0]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[10] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[10] as u32));
        t = _mm512_xor_si512(state[6], d[1]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[16] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[16] as u32));
        t = _mm512_xor_si512(state[7], d[2]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[22] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[22] as u32));
        t = _mm512_xor_si512(state[8], d[3]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[3] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[3] as u32));
        t = _mm512_xor_si512(state[9], d[4]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[9] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[9] as u32));

        state[5] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[6] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[7] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[8] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[9] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[10], d[0]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[20] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[20] as u32));
        t = _mm512_xor_si512(state[11], d[1]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[1] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[1] as u32));
        t = _mm512_xor_si512(state[12], d[2]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[7] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[7] as u32));
        t = _mm512_xor_si512(state[13], d[3]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[13] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[13] as u32));
        t = _mm512_xor_si512(state[14], d[4]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[19] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[19] as u32));

        state[10] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[11] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[12] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[13] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[14] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[15], d[0]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[5] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[5] as u32));
        t = _mm512_xor_si512(state[16], d[1]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[11] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[11] as u32));
        t = _mm512_xor_si512(state[17], d[2]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[17] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[17] as u32));
        t = _mm512_xor_si512(state[18], d[3]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[23] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[23] as u32));
        t = _mm512_xor_si512(state[19], d[4]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[4] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[4] as u32));

        state[15] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[16] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[17] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[18] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[19] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));

        t = _mm512_xor_si512(state[20], d[0]);
        c[3] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[15] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[15] as u32));
        t = _mm512_xor_si512(state[21], d[1]);
        c[4] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[21] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[21] as u32));
        t = _mm512_xor_si512(state[22], d[2]);
        c[0] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[2] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[2] as u32));
        t = _mm512_xor_si512(state[23], d[3]);
        c[1] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[8] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[8] as u32));
        t = _mm512_xor_si512(state[24], d[4]);
        c[2] = _mm512_or_si512(_mm512_slli_epi64(t, ROT[14] as u32), _mm512_srli_epi64(t, keccak::W as u32 - ROT[14] as u32));

        state[20] = _mm512_xor_si512(c[0], _mm512_andnot_si512(c[1], c[2]));
        state[21] = _mm512_xor_si512(c[1], _mm512_andnot_si512(c[2], c[3]));
        state[22] = _mm512_xor_si512(c[2], _mm512_andnot_si512(c[3], c[4]));
        state[23] = _mm512_xor_si512(c[3], _mm512_andnot_si512(c[4], c[0]));
        state[24] = _mm512_xor_si512(c[4], _mm512_andnot_si512(c[0], c[1]));
    }
}

#[cfg(test)]
mod test {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_8xkeccak_permutation() {
        use crate::keccak;
        use rand::Rng;

        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;

        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        if !is_x86_feature_detected!("avx512f") {
            return;
        }

        let mut rng = rand::rng();

        let mut keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();
        let mut keccak_statex8: [__m512i; turboshake::keccak::LANE_CNT] = keccak_state
            .iter()
            .map(|&lane| unsafe { _mm512_set1_epi64(lane as i64) })
            .collect::<Vec<__m512i>>()
            .try_into()
            .expect("Must be able to form 8x keccak-p[1600], backed by AVX512 registers");

        turboshake::keccak::permute(&mut keccak_state);
        unsafe { keccak::keccakx8::permute(&mut keccak_statex8) };

        let lane0_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 0), 0) as u64 })
            .collect::<Vec<u64>>();
        let lane1_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 0), 1) as u64 })
            .collect::<Vec<u64>>();
        let lane2_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 0), 2) as u64 })
            .collect::<Vec<u64>>();
        let lane3_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 0), 3) as u64 })
            .collect::<Vec<u64>>();
        let lane4_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 1), 0) as u64 })
            .collect::<Vec<u64>>();
        let lane5_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 1), 1) as u64 })
            .collect::<Vec<u64>>();
        let lane6_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 1), 2) as u64 })
            .collect::<Vec<u64>>();
        let lane7_keccak_state = keccak_statex8
            .iter()
            .map(|&lanex8| unsafe { _mm256_extract_epi64(_mm512_extracti64x4_epi64(lanex8, 1), 3) as u64 })
            .collect::<Vec<u64>>();

        assert_eq!(lane0_keccak_state, keccak_state);
        assert_eq!(lane1_keccak_state, keccak_state);
        assert_eq!(lane2_keccak_state, keccak_state);
        assert_eq!(lane3_keccak_state, keccak_state);
        assert_eq!(lane4_keccak_state, keccak_state);
        assert_eq!(lane5_keccak_state, keccak_state);
        assert_eq!(lane6_keccak_state, keccak_state);
        assert_eq!(lane7_keccak_state, keccak_state);
    }
}
