use crate::keccak::consts::{RC, ROT};
use turboshake::keccak;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Keccak-p\[1600\] permutation, applying 12 rounds permutation, on four states, each of dimension 5 x 5 x 64 ( = 1600 -bits ),
/// using AVX2, following <https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L527-L540>.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[allow(unused_unsafe)]
pub unsafe fn permute(state: &mut [__m256i; keccak::LANE_CNT]) {
    unsafe {
        roundx4(state, 0);
        roundx4(state, 4);
        roundx4(state, 8);
    }
}

/// Keccak-p\[1600\] round function, applying all five step mapping functions in order, for four consecutive rounds, starting from round index `ridx`.
/// This function is a line by line translation of <https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L114-L525>
/// to AVX2 compatible version, where we are permuting four Keccak-p\[1600\] permutation instances in parallel.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[allow(unused_unsafe)]
#[inline]
unsafe fn roundx4(state: &mut [__m256i; keccak::LANE_CNT], ridx: usize) {
    unsafe {
        let mut c = [_mm256_setzero_si256(); 5];
        let mut d = [_mm256_setzero_si256(); 5];
        let mut t: __m256i;

        // Round ridx + 0
        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm256_xor_si256(c[0], state[i]);
            c[1] = _mm256_xor_si256(c[1], state[i + 1]);
            c[2] = _mm256_xor_si256(c[2], state[i + 2]);
            c[3] = _mm256_xor_si256(c[3], state[i + 3]);
            c[4] = _mm256_xor_si256(c[4], state[i + 4]);
        }

        d[0] = _mm256_xor_si256(c[4], _mm256_or_si256(_mm256_slli_epi64(c[1], 1), _mm256_srli_epi64(c[1], keccak::W as i32 - 1)));
        d[1] = _mm256_xor_si256(c[0], _mm256_or_si256(_mm256_slli_epi64(c[2], 1), _mm256_srli_epi64(c[2], keccak::W as i32 - 1)));
        d[2] = _mm256_xor_si256(c[1], _mm256_or_si256(_mm256_slli_epi64(c[3], 1), _mm256_srli_epi64(c[3], keccak::W as i32 - 1)));
        d[3] = _mm256_xor_si256(c[2], _mm256_or_si256(_mm256_slli_epi64(c[4], 1), _mm256_srli_epi64(c[4], keccak::W as i32 - 1)));
        d[4] = _mm256_xor_si256(c[3], _mm256_or_si256(_mm256_slli_epi64(c[0], 1), _mm256_srli_epi64(c[0], keccak::W as i32 - 1)));

        c[0] = _mm256_xor_si256(state[0], d[0]);
        t = _mm256_xor_si256(state[6], d[1]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[6]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[6]));
        t = _mm256_xor_si256(state[12], d[2]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[12]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[12]));
        t = _mm256_xor_si256(state[18], d[3]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[18]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[18]));
        t = _mm256_xor_si256(state[24], d[4]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[24]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[24]));

        state[0] = _mm256_xor_si256(_mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2])), _mm256_set1_epi64x(RC[ridx]));
        state[6] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[12] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[18] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[24] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[10], d[0]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[10]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[10]));
        t = _mm256_xor_si256(state[16], d[1]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[16]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[16]));
        t = _mm256_xor_si256(state[22], d[2]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[22]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[22]));
        t = _mm256_xor_si256(state[3], d[3]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[3]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[3]));
        t = _mm256_xor_si256(state[9], d[4]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[9]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[9]));

        state[10] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[16] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[22] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[3] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[9] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[20], d[0]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[20]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[20]));
        t = _mm256_xor_si256(state[1], d[1]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[1]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[1]));
        t = _mm256_xor_si256(state[7], d[2]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[7]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[7]));
        t = _mm256_xor_si256(state[13], d[3]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[13]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[13]));
        t = _mm256_xor_si256(state[19], d[4]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[19]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[19]));

        state[20] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[1] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[7] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[13] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[19] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[5], d[0]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[5]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[5]));
        t = _mm256_xor_si256(state[11], d[1]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[11]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[11]));
        t = _mm256_xor_si256(state[17], d[2]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[17]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[17]));
        t = _mm256_xor_si256(state[23], d[3]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[23]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[23]));
        t = _mm256_xor_si256(state[4], d[4]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[4]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[4]));

        state[5] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[11] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[17] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[23] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[4] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[15], d[0]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[15]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[15]));
        t = _mm256_xor_si256(state[21], d[1]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[21]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[21]));
        t = _mm256_xor_si256(state[2], d[2]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[2]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[2]));
        t = _mm256_xor_si256(state[8], d[3]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[8]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[8]));
        t = _mm256_xor_si256(state[14], d[4]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[14]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[14]));

        state[15] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[21] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[2] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[8] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[14] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        // Round ridx + 1
        c.fill(_mm256_setzero_si256());

        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm256_xor_si256(c[0], state[i]);
            c[1] = _mm256_xor_si256(c[1], state[i + 1]);
            c[2] = _mm256_xor_si256(c[2], state[i + 2]);
            c[3] = _mm256_xor_si256(c[3], state[i + 3]);
            c[4] = _mm256_xor_si256(c[4], state[i + 4]);
        }

        d[0] = _mm256_xor_si256(c[4], _mm256_or_si256(_mm256_slli_epi64(c[1], 1), _mm256_srli_epi64(c[1], keccak::W as i32 - 1)));
        d[1] = _mm256_xor_si256(c[0], _mm256_or_si256(_mm256_slli_epi64(c[2], 1), _mm256_srli_epi64(c[2], keccak::W as i32 - 1)));
        d[2] = _mm256_xor_si256(c[1], _mm256_or_si256(_mm256_slli_epi64(c[3], 1), _mm256_srli_epi64(c[3], keccak::W as i32 - 1)));
        d[3] = _mm256_xor_si256(c[2], _mm256_or_si256(_mm256_slli_epi64(c[4], 1), _mm256_srli_epi64(c[4], keccak::W as i32 - 1)));
        d[4] = _mm256_xor_si256(c[3], _mm256_or_si256(_mm256_slli_epi64(c[0], 1), _mm256_srli_epi64(c[0], keccak::W as i32 - 1)));

        c[0] = _mm256_xor_si256(state[0], d[0]);
        t = _mm256_xor_si256(state[16], d[1]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[6]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[6]));
        t = _mm256_xor_si256(state[7], d[2]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[12]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[12]));
        t = _mm256_xor_si256(state[23], d[3]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[18]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[18]));
        t = _mm256_xor_si256(state[14], d[4]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[24]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[24]));

        state[0] = _mm256_xor_si256(_mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2])), _mm256_set1_epi64x(RC[ridx + 1]));
        state[16] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[7] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[23] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[14] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[20], d[0]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[10]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[10]));
        t = _mm256_xor_si256(state[11], d[1]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[16]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[16]));
        t = _mm256_xor_si256(state[2], d[2]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[22]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[22]));
        t = _mm256_xor_si256(state[18], d[3]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[3]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[3]));
        t = _mm256_xor_si256(state[9], d[4]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[9]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[9]));

        state[20] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[11] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[2] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[18] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[9] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[15], d[0]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[20]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[20]));
        t = _mm256_xor_si256(state[6], d[1]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[1]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[1]));
        t = _mm256_xor_si256(state[22], d[2]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[7]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[7]));
        t = _mm256_xor_si256(state[13], d[3]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[13]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[13]));
        t = _mm256_xor_si256(state[4], d[4]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[19]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[19]));

        state[15] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[6] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[22] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[13] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[4] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[10], d[0]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[5]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[5]));
        t = _mm256_xor_si256(state[1], d[1]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[11]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[11]));
        t = _mm256_xor_si256(state[17], d[2]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[17]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[17]));
        t = _mm256_xor_si256(state[8], d[3]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[23]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[23]));
        t = _mm256_xor_si256(state[24], d[4]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[4]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[4]));

        state[10] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[1] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[17] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[8] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[24] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[5], d[0]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[15]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[15]));
        t = _mm256_xor_si256(state[21], d[1]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[21]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[21]));
        t = _mm256_xor_si256(state[12], d[2]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[2]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[2]));
        t = _mm256_xor_si256(state[3], d[3]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[8]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[8]));
        t = _mm256_xor_si256(state[19], d[4]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[14]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[14]));

        state[5] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[21] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[12] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[3] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[19] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        // Round ridx + 2
        c.fill(_mm256_setzero_si256());

        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm256_xor_si256(c[0], state[i]);
            c[1] = _mm256_xor_si256(c[1], state[i + 1]);
            c[2] = _mm256_xor_si256(c[2], state[i + 2]);
            c[3] = _mm256_xor_si256(c[3], state[i + 3]);
            c[4] = _mm256_xor_si256(c[4], state[i + 4]);
        }

        d[0] = _mm256_xor_si256(c[4], _mm256_or_si256(_mm256_slli_epi64(c[1], 1), _mm256_srli_epi64(c[1], keccak::W as i32 - 1)));
        d[1] = _mm256_xor_si256(c[0], _mm256_or_si256(_mm256_slli_epi64(c[2], 1), _mm256_srli_epi64(c[2], keccak::W as i32 - 1)));
        d[2] = _mm256_xor_si256(c[1], _mm256_or_si256(_mm256_slli_epi64(c[3], 1), _mm256_srli_epi64(c[3], keccak::W as i32 - 1)));
        d[3] = _mm256_xor_si256(c[2], _mm256_or_si256(_mm256_slli_epi64(c[4], 1), _mm256_srli_epi64(c[4], keccak::W as i32 - 1)));
        d[4] = _mm256_xor_si256(c[3], _mm256_or_si256(_mm256_slli_epi64(c[0], 1), _mm256_srli_epi64(c[0], keccak::W as i32 - 1)));

        c[0] = _mm256_xor_si256(state[0], d[0]);
        t = _mm256_xor_si256(state[11], d[1]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[6]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[6]));
        t = _mm256_xor_si256(state[22], d[2]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[12]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[12]));
        t = _mm256_xor_si256(state[8], d[3]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[18]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[18]));
        t = _mm256_xor_si256(state[19], d[4]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[24]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[24]));

        state[0] = _mm256_xor_si256(_mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2])), _mm256_set1_epi64x(RC[ridx + 2]));
        state[11] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[22] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[8] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[19] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[15], d[0]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[10]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[10]));
        t = _mm256_xor_si256(state[1], d[1]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[16]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[16]));
        t = _mm256_xor_si256(state[12], d[2]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[22]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[22]));
        t = _mm256_xor_si256(state[23], d[3]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[3]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[3]));
        t = _mm256_xor_si256(state[9], d[4]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[9]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[9]));

        state[15] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[1] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[12] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[23] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[9] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[5], d[0]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[20]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[20]));
        t = _mm256_xor_si256(state[16], d[1]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[1]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[1]));
        t = _mm256_xor_si256(state[2], d[2]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[7]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[7]));
        t = _mm256_xor_si256(state[13], d[3]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[13]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[13]));
        t = _mm256_xor_si256(state[24], d[4]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[19]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[19]));

        state[5] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[16] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[2] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[13] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[24] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[20], d[0]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[5]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[5]));
        t = _mm256_xor_si256(state[6], d[1]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[11]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[11]));
        t = _mm256_xor_si256(state[17], d[2]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[17]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[17]));
        t = _mm256_xor_si256(state[3], d[3]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[23]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[23]));
        t = _mm256_xor_si256(state[14], d[4]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[4]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[4]));

        state[20] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[6] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[17] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[3] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[14] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[10], d[0]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[15]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[15]));
        t = _mm256_xor_si256(state[21], d[1]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[21]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[21]));
        t = _mm256_xor_si256(state[7], d[2]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[2]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[2]));
        t = _mm256_xor_si256(state[18], d[3]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[8]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[8]));
        t = _mm256_xor_si256(state[4], d[4]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[14]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[14]));

        state[10] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[21] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[7] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[18] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[4] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        // Round ridx + 3
        c.fill(_mm256_setzero_si256());

        for i in (0..keccak::LANE_CNT).step_by(5) {
            c[0] = _mm256_xor_si256(c[0], state[i]);
            c[1] = _mm256_xor_si256(c[1], state[i + 1]);
            c[2] = _mm256_xor_si256(c[2], state[i + 2]);
            c[3] = _mm256_xor_si256(c[3], state[i + 3]);
            c[4] = _mm256_xor_si256(c[4], state[i + 4]);
        }

        d[0] = _mm256_xor_si256(c[4], _mm256_or_si256(_mm256_slli_epi64(c[1], 1), _mm256_srli_epi64(c[1], keccak::W as i32 - 1)));
        d[1] = _mm256_xor_si256(c[0], _mm256_or_si256(_mm256_slli_epi64(c[2], 1), _mm256_srli_epi64(c[2], keccak::W as i32 - 1)));
        d[2] = _mm256_xor_si256(c[1], _mm256_or_si256(_mm256_slli_epi64(c[3], 1), _mm256_srli_epi64(c[3], keccak::W as i32 - 1)));
        d[3] = _mm256_xor_si256(c[2], _mm256_or_si256(_mm256_slli_epi64(c[4], 1), _mm256_srli_epi64(c[4], keccak::W as i32 - 1)));
        d[4] = _mm256_xor_si256(c[3], _mm256_or_si256(_mm256_slli_epi64(c[0], 1), _mm256_srli_epi64(c[0], keccak::W as i32 - 1)));

        c[0] = _mm256_xor_si256(state[0], d[0]);
        t = _mm256_xor_si256(state[1], d[1]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[6]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[6]));
        t = _mm256_xor_si256(state[2], d[2]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[12]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[12]));
        t = _mm256_xor_si256(state[3], d[3]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[18]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[18]));
        t = _mm256_xor_si256(state[4], d[4]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[24]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[24]));

        state[0] = _mm256_xor_si256(_mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2])), _mm256_set1_epi64x(RC[ridx + 3]));
        state[1] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[2] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[3] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[4] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[5], d[0]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[10]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[10]));
        t = _mm256_xor_si256(state[6], d[1]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[16]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[16]));
        t = _mm256_xor_si256(state[7], d[2]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[22]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[22]));
        t = _mm256_xor_si256(state[8], d[3]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[3]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[3]));
        t = _mm256_xor_si256(state[9], d[4]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[9]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[9]));

        state[5] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[6] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[7] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[8] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[9] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[10], d[0]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[20]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[20]));
        t = _mm256_xor_si256(state[11], d[1]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[1]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[1]));
        t = _mm256_xor_si256(state[12], d[2]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[7]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[7]));
        t = _mm256_xor_si256(state[13], d[3]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[13]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[13]));
        t = _mm256_xor_si256(state[14], d[4]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[19]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[19]));

        state[10] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[11] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[12] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[13] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[14] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[15], d[0]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[5]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[5]));
        t = _mm256_xor_si256(state[16], d[1]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[11]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[11]));
        t = _mm256_xor_si256(state[17], d[2]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[17]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[17]));
        t = _mm256_xor_si256(state[18], d[3]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[23]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[23]));
        t = _mm256_xor_si256(state[19], d[4]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[4]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[4]));

        state[15] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[16] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[17] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[18] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[19] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));

        t = _mm256_xor_si256(state[20], d[0]);
        c[3] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[15]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[15]));
        t = _mm256_xor_si256(state[21], d[1]);
        c[4] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[21]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[21]));
        t = _mm256_xor_si256(state[22], d[2]);
        c[0] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[2]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[2]));
        t = _mm256_xor_si256(state[23], d[3]);
        c[1] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[8]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[8]));
        t = _mm256_xor_si256(state[24], d[4]);
        c[2] = _mm256_or_si256(_mm256_slli_epi64(t, ROT[14]), _mm256_srli_epi64(t, keccak::W as i32 - ROT[14]));

        state[20] = _mm256_xor_si256(c[0], _mm256_andnot_si256(c[1], c[2]));
        state[21] = _mm256_xor_si256(c[1], _mm256_andnot_si256(c[2], c[3]));
        state[22] = _mm256_xor_si256(c[2], _mm256_andnot_si256(c[3], c[4]));
        state[23] = _mm256_xor_si256(c[3], _mm256_andnot_si256(c[4], c[0]));
        state[24] = _mm256_xor_si256(c[4], _mm256_andnot_si256(c[0], c[1]));
    }
}

#[cfg(test)]
mod test {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_4xkeccak_permutation() {
        use crate::keccak;
        use rand::Rng;

        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;

        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut rng = rand::rng();

        let mut keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();
        let mut keccak_statex4: [__m256i; turboshake::keccak::LANE_CNT] = keccak_state
            .iter()
            .map(|&lane| unsafe { _mm256_set1_epi64x(lane as i64) })
            .collect::<Vec<__m256i>>()
            .try_into()
            .expect("Must be able to form 4x keccak-p[1600], backed by AVX2 registers");

        turboshake::keccak::permute(&mut keccak_state);
        unsafe { keccak::keccakx4::permute(&mut keccak_statex4) };

        let lane0_keccak_state = keccak_statex4
            .iter()
            .map(|&lanex2| unsafe { _mm256_extract_epi64(lanex2, 0) as u64 })
            .collect::<Vec<u64>>();
        let lane1_keccak_state = keccak_statex4
            .iter()
            .map(|&lanex2| unsafe { _mm256_extract_epi64(lanex2, 1) as u64 })
            .collect::<Vec<u64>>();
        let lane2_keccak_state = keccak_statex4
            .iter()
            .map(|&lanex2| unsafe { _mm256_extract_epi64(lanex2, 2) as u64 })
            .collect::<Vec<u64>>();
        let lane3_keccak_state = keccak_statex4
            .iter()
            .map(|&lanex2| unsafe { _mm256_extract_epi64(lanex2, 3) as u64 })
            .collect::<Vec<u64>>();

        assert_eq!(lane0_keccak_state, keccak_state);
        assert_eq!(lane1_keccak_state, keccak_state);
        assert_eq!(lane2_keccak_state, keccak_state);
        assert_eq!(lane3_keccak_state, keccak_state);
    }
}
