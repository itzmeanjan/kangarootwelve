use crate::cv::consts::CHUNK_BYTE_LEN;
use crate::keccak::keccakx2;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
#[allow(unused_unsafe)]
pub fn compute_chaining_valuex2<const NUM_RATE_BITS: usize, const DOMAIN_SEPARATOR: u8, const CV_SIZE: usize>(chunk: &[u8], chaining_valuex2: &mut [u8]) {
    unsafe {
        let num_rate_bytes = NUM_RATE_BITS / u8::BITS as usize;
        let num_rate_words = NUM_RATE_BITS / turboshake::keccak::W;

        let num_bytes_in_last_block = CHUNK_BYTE_LEN % num_rate_bytes;
        let num_words_in_last_block = num_bytes_in_last_block / u8::BITS as usize;

        let mut keccak_statex2 = [_mm_setzero_si128(); turboshake::keccak::LANE_CNT];

        let (chunk0, chunk1) = chunk.split_at(CHUNK_BYTE_LEN);

        let mut chunk0_iter = chunk0.chunks_exact(num_rate_bytes);
        let mut chunk1_iter = chunk1.chunks_exact(num_rate_bytes);

        chunk0_iter.by_ref().zip(chunk1_iter.by_ref()).for_each(|(block0, block1)| {
            block0
                .chunks_exact(u8::BITS as usize)
                .zip(block1.chunks_exact(u8::BITS as usize))
                .map(|(word0, word1)| {
                    _mm_xor_si128(
                        _mm_slli_si128(_mm_loadu_si64(word1.as_ptr() as *const _), 8),
                        _mm_loadu_si64(word0.as_ptr() as *const _),
                    )
                })
                .zip(keccak_statex2[..num_rate_words].iter_mut())
                .for_each(|(wordx2, keccak_state_wordx2)| {
                    *keccak_state_wordx2 = _mm_xor_si128(*keccak_state_wordx2, wordx2);
                });

            keccakx2::permute(&mut keccak_statex2);
        });

        let last_block0 = chunk0_iter.remainder();
        let last_block1 = chunk1_iter.remainder();

        last_block0
            .chunks_exact(u8::BITS as usize)
            .zip(last_block1.chunks_exact(u8::BITS as usize))
            .map(|(word0, word1)| {
                _mm_xor_si128(
                    _mm_slli_si128(_mm_loadu_si64(word1.as_ptr() as *const _), 8),
                    _mm_loadu_si64(word0.as_ptr() as *const _),
                )
            })
            .zip(keccak_statex2[..num_words_in_last_block].iter_mut())
            .for_each(|(wordx2, keccak_state_wordx2)| {
                *keccak_state_wordx2 = _mm_xor_si128(*keccak_state_wordx2, wordx2);
            });

        let ds_wordx2 = _mm_set1_epi64x(DOMAIN_SEPARATOR as i64);
        keccak_statex2[num_words_in_last_block] = _mm_xor_si128(keccak_statex2[num_words_in_last_block], ds_wordx2);

        let padding_word = 0x80u64 << (turboshake::keccak::W - u8::BITS as usize);
        let padding_wordx2 = _mm_set1_epi64x(padding_word as i64);
        keccak_statex2[num_rate_words - 1] = _mm_xor_si128(keccak_statex2[num_rate_words - 1], padding_wordx2);

        keccakx2::permute(&mut keccak_statex2);

        let (cv0, cv1) = chaining_valuex2.split_at_mut(CV_SIZE);

        cv0.chunks_exact_mut(u8::BITS as usize)
            .zip(cv1.chunks_exact_mut(u8::BITS as usize))
            .enumerate()
            .for_each(|(word_idx, (cv0_word_bytes, cv1_word_bytes))| {
                let keccak_state_wordx2 = keccak_statex2[word_idx];

                _mm_storeu_si64(cv0_word_bytes.as_mut_ptr() as *mut _, keccak_state_wordx2);
                _mm_storeu_si64(cv1_word_bytes.as_mut_ptr() as *mut _, _mm_srli_si128(keccak_state_wordx2, 8));
            });
    }
}

#[cfg(test)]
mod test {
    use crate::cv::consts::CHUNK_BYTE_LEN;
    use turboshake::{TurboShake128, TurboShake256};

    const TS128_NUM_RATE_BITS: usize = 1600 - 256;
    const TS256_NUM_RATE_BITS: usize = 1600 - 512;

    const TS128_CHAINING_VALUE_BYTE_LEN: usize = 32;
    const TS256_CHAINING_VALUE_BYTE_LEN: usize = 64;

    const DOMAIN_SEPARATOR: u8 = 0x0b;

    fn compute_cv_with_ts128(chunk: &[u8; CHUNK_BYTE_LEN]) -> [u8; TS128_CHAINING_VALUE_BYTE_LEN] {
        let mut cv = [0u8; TS128_CHAINING_VALUE_BYTE_LEN];
        let mut ts128 = TurboShake128::default();

        let _ = ts128.absorb(chunk);
        let _ = ts128.finalize::<DOMAIN_SEPARATOR>();
        let _ = ts128.squeeze(&mut cv);

        cv
    }

    fn compute_cv_with_ts256(chunk: &[u8; CHUNK_BYTE_LEN]) -> [u8; TS256_CHAINING_VALUE_BYTE_LEN] {
        let mut cv = [0u8; TS256_CHAINING_VALUE_BYTE_LEN];
        let mut ts256 = TurboShake256::default();

        let _ = ts256.absorb(chunk);
        let _ = ts256.finalize::<DOMAIN_SEPARATOR>();
        let _ = ts256.squeeze(&mut cv);

        cv
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_2xcompute_chaining_value_with_ts128() {
        use crate::cv::cvx2::compute_chaining_valuex2;
        use rand::Rng;

        if !is_x86_feature_detected!("sse2") {
            return;
        }

        let mut rng = rand::rng();

        let chunk: [u8; 2 * CHUNK_BYTE_LEN] = rng.random();
        let (chunk0, chunk1) = chunk.split_at(CHUNK_BYTE_LEN);

        let expected_cv0 = compute_cv_with_ts128(chunk0.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv1 = compute_cv_with_ts128(chunk1.try_into().expect("must not fail to convert slice to array reference"));

        let mut computed_cv = [0u8; 2 * TS128_CHAINING_VALUE_BYTE_LEN];
        unsafe { compute_chaining_valuex2::<TS128_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS128_CHAINING_VALUE_BYTE_LEN>(&chunk, &mut computed_cv) };

        assert_eq!(expected_cv0, computed_cv[..TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv1, computed_cv[TS128_CHAINING_VALUE_BYTE_LEN..]);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_2xcompute_chaining_value_with_ts256() {
        use crate::cv::cvx2::compute_chaining_valuex2;
        use rand::Rng;

        if !is_x86_feature_detected!("sse2") {
            return;
        }

        let mut rng = rand::rng();

        let chunk: [u8; 2 * CHUNK_BYTE_LEN] = rng.random();
        let (chunk0, chunk1) = chunk.split_at(CHUNK_BYTE_LEN);

        let expected_cv0 = compute_cv_with_ts256(chunk0.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv1 = compute_cv_with_ts256(chunk1.try_into().expect("must not fail to convert slice to array reference"));

        let mut computed_cv = [0u8; 2 * TS256_CHAINING_VALUE_BYTE_LEN];
        unsafe { compute_chaining_valuex2::<TS256_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS256_CHAINING_VALUE_BYTE_LEN>(&chunk, &mut computed_cv) };

        assert_eq!(expected_cv0, computed_cv[..TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv1, computed_cv[TS256_CHAINING_VALUE_BYTE_LEN..]);
    }
}
