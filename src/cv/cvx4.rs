use crate::cv::consts::CHUNK_BYTE_LEN;
use crate::keccak::keccakx4;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const CHUNKX4_BYTE_LEN: usize = 4 * CHUNK_BYTE_LEN;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[allow(unused_unsafe)]
pub fn compute_chaining_valuex4<const NUM_RATE_BITS: usize, const DOMAIN_SEPARATOR: u8, const CV_SIZE: usize>(
    chunkx4: &[u8; CHUNKX4_BYTE_LEN],
    chaining_valuex4: &mut [u8],
) {
    unsafe {
        let num_rate_bytes = NUM_RATE_BITS / u8::BITS as usize;
        let num_rate_words = NUM_RATE_BITS / turboshake::keccak::W;

        let num_bytes_in_last_block = CHUNK_BYTE_LEN % num_rate_bytes;
        let num_words_in_last_block = num_bytes_in_last_block / u8::BITS as usize;

        let mut keccak_statex4 = [_mm256_setzero_si256(); turboshake::keccak::LANE_CNT];

        let (chunk0, chunk1, chunk2, chunk3) = {
            let (left, right) = chunkx4.split_at(2 * CHUNK_BYTE_LEN);

            let (chunk0, chunk1) = left.split_at(CHUNK_BYTE_LEN);
            let (chunk2, chunk3) = right.split_at(CHUNK_BYTE_LEN);

            (chunk0, chunk1, chunk2, chunk3)
        };

        let mut chunk0_iter = chunk0.chunks_exact(num_rate_bytes);
        let mut chunk1_iter = chunk1.chunks_exact(num_rate_bytes);
        let mut chunk2_iter = chunk2.chunks_exact(num_rate_bytes);
        let mut chunk3_iter = chunk3.chunks_exact(num_rate_bytes);

        chunk0_iter
            .by_ref()
            .zip(chunk1_iter.by_ref())
            .zip(chunk2_iter.by_ref())
            .zip(chunk3_iter.by_ref())
            .for_each(|(((block0, block1), block2), block3)| {
                block0
                    .chunks_exact(u8::BITS as usize)
                    .zip(block1.chunks_exact(u8::BITS as usize))
                    .zip(block2.chunks_exact(u8::BITS as usize))
                    .zip(block3.chunks_exact(u8::BITS as usize))
                    .map(|(((word0, word1), word2), word3)| unsafe {
                        _mm256_set_epi64x(
                            u64::from_le_bytes(word3.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word2.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word1.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word0.try_into().unwrap_unchecked()) as i64,
                        )
                    })
                    .zip(keccak_statex4[..num_rate_words].iter_mut())
                    .for_each(|(wordx4, keccak_state_wordx4)| {
                        *keccak_state_wordx4 = _mm256_xor_si256(*keccak_state_wordx4, wordx4);
                    });

                keccakx4::permute(&mut keccak_statex4);
            });

        let last_block0 = chunk0_iter.remainder();
        let last_block1 = chunk1_iter.remainder();
        let last_block2 = chunk2_iter.remainder();
        let last_block3 = chunk3_iter.remainder();

        last_block0
            .chunks_exact(u8::BITS as usize)
            .zip(last_block1.chunks_exact(u8::BITS as usize))
            .zip(last_block2.chunks_exact(u8::BITS as usize))
            .zip(last_block3.chunks_exact(u8::BITS as usize))
            .map(|(((word0, word1), word2), word3)| {
                _mm256_set_epi64x(
                    u64::from_le_bytes(word3.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word2.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word1.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word0.try_into().unwrap_unchecked()) as i64,
                )
            })
            .zip(keccak_statex4[..num_words_in_last_block].iter_mut())
            .for_each(|(wordx4, keccak_state_wordx4)| {
                *keccak_state_wordx4 = _mm256_xor_si256(*keccak_state_wordx4, wordx4);
            });

        let ds_wordx4 = _mm256_set1_epi64x(DOMAIN_SEPARATOR as i64);
        keccak_statex4[num_words_in_last_block] = _mm256_xor_si256(keccak_statex4[num_words_in_last_block], ds_wordx4);

        let padding_word = 0x80u64 << (turboshake::keccak::W - u8::BITS as usize);
        let padding_wordx4 = _mm256_set1_epi64x(padding_word as i64);
        keccak_statex4[num_rate_words - 1] = _mm256_xor_si256(keccak_statex4[num_rate_words - 1], padding_wordx4);

        keccakx4::permute(&mut keccak_statex4);

        let (cv0, cv1, cv2, cv3) = {
            let (left, right) = chaining_valuex4.split_at_mut(2 * CV_SIZE);

            let (cv0, cv1) = left.split_at_mut(CV_SIZE);
            let (cv2, cv3) = right.split_at_mut(CV_SIZE);

            (cv0, cv1, cv2, cv3)
        };

        cv0.chunks_exact_mut(u8::BITS as usize)
            .zip(cv1.chunks_exact_mut(u8::BITS as usize))
            .zip(cv2.chunks_exact_mut(u8::BITS as usize))
            .zip(cv3.chunks_exact_mut(u8::BITS as usize))
            .enumerate()
            .for_each(|(word_idx, (((cv0_word_bytes, cv1_word_bytes), cv2_word_bytes), cv3_word_bytes))| {
                let keccak_state_wordx4 = keccak_statex4[word_idx];

                let mut wordx4_as_bytes = [0u8; 4 * u8::BITS as usize];
                _mm256_storeu_si256(wordx4_as_bytes.as_mut_ptr() as *mut _, keccak_state_wordx4);

                cv0_word_bytes.copy_from_slice(&wordx4_as_bytes[..8]);
                cv1_word_bytes.copy_from_slice(&wordx4_as_bytes[8..16]);
                cv2_word_bytes.copy_from_slice(&wordx4_as_bytes[16..24]);
                cv3_word_bytes.copy_from_slice(&wordx4_as_bytes[24..]);
            });
    }
}

#[cfg(test)]
mod test {
    use crate::cv::consts::CHUNK_BYTE_LEN;
    use crate::cv::cvx4;
    use rand::Rng;
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
    fn test_4xcompute_chaining_value_with_ts128() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut rng = rand::rng();

        let chunk: [u8; 4 * CHUNK_BYTE_LEN] = rng.random();
        let (chunk0, chunk1, chunk2, chunk3) = {
            let (left, right) = chunk.split_at(2 * CHUNK_BYTE_LEN);

            let (chunk0, chunk1) = left.split_at(CHUNK_BYTE_LEN);
            let (chunk2, chunk3) = right.split_at(CHUNK_BYTE_LEN);

            (chunk0, chunk1, chunk2, chunk3)
        };

        let expected_cv0 = compute_cv_with_ts128(chunk0.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv1 = compute_cv_with_ts128(chunk1.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv2 = compute_cv_with_ts128(chunk2.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv3 = compute_cv_with_ts128(chunk3.try_into().expect("must not fail to convert slice to array reference"));

        let mut computed_cv = [0u8; 4 * TS128_CHAINING_VALUE_BYTE_LEN];
        unsafe { cvx4::compute_chaining_valuex4::<TS128_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS128_CHAINING_VALUE_BYTE_LEN>(&chunk, &mut computed_cv) };

        assert_eq!(expected_cv0, computed_cv[..TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv1, computed_cv[TS128_CHAINING_VALUE_BYTE_LEN..2 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv2, computed_cv[2 * TS128_CHAINING_VALUE_BYTE_LEN..3 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv3, computed_cv[3 * TS128_CHAINING_VALUE_BYTE_LEN..]);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_4xcompute_chaining_value_with_ts256() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut rng = rand::rng();

        let chunk: [u8; 4 * CHUNK_BYTE_LEN] = rng.random();
        let (chunk0, chunk1, chunk2, chunk3) = {
            let (left, right) = chunk.split_at(2 * CHUNK_BYTE_LEN);

            let (chunk0, chunk1) = left.split_at(CHUNK_BYTE_LEN);
            let (chunk2, chunk3) = right.split_at(CHUNK_BYTE_LEN);

            (chunk0, chunk1, chunk2, chunk3)
        };

        let expected_cv0 = compute_cv_with_ts256(chunk0.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv1 = compute_cv_with_ts256(chunk1.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv2 = compute_cv_with_ts256(chunk2.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv3 = compute_cv_with_ts256(chunk3.try_into().expect("must not fail to convert slice to array reference"));

        let mut computed_cv = [0u8; 4 * TS256_CHAINING_VALUE_BYTE_LEN];
        unsafe { cvx4::compute_chaining_valuex4::<TS256_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS256_CHAINING_VALUE_BYTE_LEN>(&chunk, &mut computed_cv) };

        assert_eq!(expected_cv0, computed_cv[..TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv1, computed_cv[TS256_CHAINING_VALUE_BYTE_LEN..2 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv2, computed_cv[2 * TS256_CHAINING_VALUE_BYTE_LEN..3 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv3, computed_cv[3 * TS256_CHAINING_VALUE_BYTE_LEN..]);
    }
}
