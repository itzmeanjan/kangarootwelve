use crate::cv::consts::CHUNK_BYTE_LEN;
use crate::keccak::keccakx8;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const CHUNKX8_BYTE_LEN: usize = 8 * CHUNK_BYTE_LEN;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
#[allow(unused_unsafe)]
pub unsafe fn compute_chaining_valuex8<const NUM_RATE_BITS: usize, const DOMAIN_SEPARATOR: u8, const CV_SIZE: usize>(
    chunkx8: &[u8; CHUNKX8_BYTE_LEN],
    chaining_valuex8: &mut [u8],
) {
    unsafe {
        let num_rate_bytes = NUM_RATE_BITS / u8::BITS as usize;
        let num_rate_words = NUM_RATE_BITS / turboshake::keccak::W;

        let num_bytes_in_last_block = CHUNK_BYTE_LEN % num_rate_bytes;
        let num_words_in_last_block = num_bytes_in_last_block / u8::BITS as usize;

        let mut keccak_statex8 = [_mm512_setzero_si512(); turboshake::keccak::LANE_CNT];

        let (chunk0, chunk1, chunk2, chunk3, chunk4, chunk5, chunk6, chunk7) = {
            let (chunk0, rest) = chunkx8.split_at(CHUNK_BYTE_LEN);
            let (chunk1, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk2, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk3, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk4, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk5, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk6, chunk7) = rest.split_at(CHUNK_BYTE_LEN);

            (chunk0, chunk1, chunk2, chunk3, chunk4, chunk5, chunk6, chunk7)
        };

        let mut chunk0_iter = chunk0.chunks_exact(num_rate_bytes);
        let mut chunk1_iter = chunk1.chunks_exact(num_rate_bytes);
        let mut chunk2_iter = chunk2.chunks_exact(num_rate_bytes);
        let mut chunk3_iter = chunk3.chunks_exact(num_rate_bytes);
        let mut chunk4_iter = chunk4.chunks_exact(num_rate_bytes);
        let mut chunk5_iter = chunk5.chunks_exact(num_rate_bytes);
        let mut chunk6_iter = chunk6.chunks_exact(num_rate_bytes);
        let mut chunk7_iter = chunk7.chunks_exact(num_rate_bytes);

        chunk0_iter
            .by_ref()
            .zip(chunk1_iter.by_ref())
            .zip(chunk2_iter.by_ref())
            .zip(chunk3_iter.by_ref())
            .zip(chunk4_iter.by_ref())
            .zip(chunk5_iter.by_ref())
            .zip(chunk6_iter.by_ref())
            .zip(chunk7_iter.by_ref())
            .for_each(|(((((((block0, block1), block2), block3), block4), block5), block6), block7)| {
                block0
                    .chunks_exact(u8::BITS as usize)
                    .zip(block1.chunks_exact(u8::BITS as usize))
                    .zip(block2.chunks_exact(u8::BITS as usize))
                    .zip(block3.chunks_exact(u8::BITS as usize))
                    .zip(block4.chunks_exact(u8::BITS as usize))
                    .zip(block5.chunks_exact(u8::BITS as usize))
                    .zip(block6.chunks_exact(u8::BITS as usize))
                    .zip(block7.chunks_exact(u8::BITS as usize))
                    .map(|(((((((word0, word1), word2), word3), word4), word5), word6), word7)| unsafe {
                        _mm512_set_epi64(
                            u64::from_le_bytes(word7.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word6.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word5.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word4.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word3.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word2.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word1.try_into().unwrap_unchecked()) as i64,
                            u64::from_le_bytes(word0.try_into().unwrap_unchecked()) as i64,
                        )
                    })
                    .zip(keccak_statex8[..num_rate_words].iter_mut())
                    .for_each(|(wordx8, keccak_state_wordx8)| {
                        *keccak_state_wordx8 = _mm512_xor_si512(*keccak_state_wordx8, wordx8);
                    });

                keccakx8::permute(&mut keccak_statex8);
            });

        let last_block0 = chunk0_iter.remainder();
        let last_block1 = chunk1_iter.remainder();
        let last_block2 = chunk2_iter.remainder();
        let last_block3 = chunk3_iter.remainder();
        let last_block4 = chunk4_iter.remainder();
        let last_block5 = chunk5_iter.remainder();
        let last_block6 = chunk6_iter.remainder();
        let last_block7 = chunk7_iter.remainder();

        last_block0
            .chunks_exact(u8::BITS as usize)
            .zip(last_block1.chunks_exact(u8::BITS as usize))
            .zip(last_block2.chunks_exact(u8::BITS as usize))
            .zip(last_block3.chunks_exact(u8::BITS as usize))
            .zip(last_block4.chunks_exact(u8::BITS as usize))
            .zip(last_block5.chunks_exact(u8::BITS as usize))
            .zip(last_block6.chunks_exact(u8::BITS as usize))
            .zip(last_block7.chunks_exact(u8::BITS as usize))
            .map(|(((((((word0, word1), word2), word3), word4), word5), word6), word7)| {
                _mm512_set_epi64(
                    u64::from_le_bytes(word7.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word6.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word5.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word4.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word3.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word2.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word1.try_into().unwrap_unchecked()) as i64,
                    u64::from_le_bytes(word0.try_into().unwrap_unchecked()) as i64,
                )
            })
            .zip(keccak_statex8[..num_words_in_last_block].iter_mut())
            .for_each(|(wordx8, keccak_state_wordx8)| {
                *keccak_state_wordx8 = _mm512_xor_si512(*keccak_state_wordx8, wordx8);
            });

        let ds_wordx8 = _mm512_set1_epi64(DOMAIN_SEPARATOR as i64);
        keccak_statex8[num_words_in_last_block] = _mm512_xor_si512(keccak_statex8[num_words_in_last_block], ds_wordx8);

        let padding_word = 0x80u64 << (turboshake::keccak::W - u8::BITS as usize);
        let padding_wordx8 = _mm512_set1_epi64(padding_word as i64);
        keccak_statex8[num_rate_words - 1] = _mm512_xor_si512(keccak_statex8[num_rate_words - 1], padding_wordx8);

        keccakx8::permute(&mut keccak_statex8);

        let (cv0, cv1, cv2, cv3, cv4, cv5, cv6, cv7) = {
            let (cv0, rest) = chaining_valuex8.split_at_mut(CV_SIZE);
            let (cv1, rest) = rest.split_at_mut(CV_SIZE);
            let (cv2, rest) = rest.split_at_mut(CV_SIZE);
            let (cv3, rest) = rest.split_at_mut(CV_SIZE);
            let (cv4, rest) = rest.split_at_mut(CV_SIZE);
            let (cv5, rest) = rest.split_at_mut(CV_SIZE);
            let (cv6, cv7) = rest.split_at_mut(CV_SIZE);

            (cv0, cv1, cv2, cv3, cv4, cv5, cv6, cv7)
        };

        cv0.chunks_exact_mut(u8::BITS as usize)
            .zip(cv1.chunks_exact_mut(u8::BITS as usize))
            .zip(cv2.chunks_exact_mut(u8::BITS as usize))
            .zip(cv3.chunks_exact_mut(u8::BITS as usize))
            .zip(cv4.chunks_exact_mut(u8::BITS as usize))
            .zip(cv5.chunks_exact_mut(u8::BITS as usize))
            .zip(cv6.chunks_exact_mut(u8::BITS as usize))
            .zip(cv7.chunks_exact_mut(u8::BITS as usize))
            .enumerate()
            .for_each(
                |(
                    word_idx,
                    (
                        ((((((cv0_word_bytes, cv1_word_bytes), cv2_word_bytes), cv3_word_bytes), cv4_word_bytes), cv5_word_bytes), cv6_word_bytes),
                        cv7_word_bytes,
                    ),
                )| {
                    let keccak_state_wordx8 = keccak_statex8[word_idx];

                    let mut wordx8_as_bytes = [0u8; 8 * u8::BITS as usize];
                    _mm512_storeu_si512(wordx8_as_bytes.as_mut_ptr() as *mut _, keccak_state_wordx8);

                    cv0_word_bytes.copy_from_slice(&wordx8_as_bytes[..8]);
                    cv1_word_bytes.copy_from_slice(&wordx8_as_bytes[8..16]);
                    cv2_word_bytes.copy_from_slice(&wordx8_as_bytes[16..24]);
                    cv3_word_bytes.copy_from_slice(&wordx8_as_bytes[24..32]);
                    cv4_word_bytes.copy_from_slice(&wordx8_as_bytes[32..40]);
                    cv5_word_bytes.copy_from_slice(&wordx8_as_bytes[40..48]);
                    cv6_word_bytes.copy_from_slice(&wordx8_as_bytes[48..56]);
                    cv7_word_bytes.copy_from_slice(&wordx8_as_bytes[56..]);
                },
            );
    }
}

#[cfg(test)]
mod test {
    use crate::cv::consts::CHUNK_BYTE_LEN;
    use crate::cv::cvx8;
    use crate::cv::cvx8::CHUNKX8_BYTE_LEN;
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
    fn test_8xcompute_chaining_value_with_ts128() {
        if !is_x86_feature_detected!("avx512f") {
            return;
        }

        let mut rng = rand::rng();
        let chunkx8: [u8; CHUNKX8_BYTE_LEN] = rng.random();

        let (chunk0, chunk1, chunk2, chunk3, chunk4, chunk5, chunk6, chunk7) = {
            let (chunk0, rest) = chunkx8.split_at(CHUNK_BYTE_LEN);
            let (chunk1, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk2, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk3, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk4, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk5, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk6, chunk7) = rest.split_at(CHUNK_BYTE_LEN);

            (chunk0, chunk1, chunk2, chunk3, chunk4, chunk5, chunk6, chunk7)
        };

        let expected_cv0 = compute_cv_with_ts128(chunk0.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv1 = compute_cv_with_ts128(chunk1.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv2 = compute_cv_with_ts128(chunk2.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv3 = compute_cv_with_ts128(chunk3.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv4 = compute_cv_with_ts128(chunk4.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv5 = compute_cv_with_ts128(chunk5.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv6 = compute_cv_with_ts128(chunk6.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv7 = compute_cv_with_ts128(chunk7.try_into().expect("must not fail to convert slice to array reference"));

        let mut computed_cv = [0u8; 8 * TS128_CHAINING_VALUE_BYTE_LEN];
        unsafe { cvx8::compute_chaining_valuex8::<TS128_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS128_CHAINING_VALUE_BYTE_LEN>(&chunkx8, &mut computed_cv) };

        assert_eq!(expected_cv0, computed_cv[..TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv1, computed_cv[TS128_CHAINING_VALUE_BYTE_LEN..2 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv2, computed_cv[2 * TS128_CHAINING_VALUE_BYTE_LEN..3 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv3, computed_cv[3 * TS128_CHAINING_VALUE_BYTE_LEN..4 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv4, computed_cv[4 * TS128_CHAINING_VALUE_BYTE_LEN..5 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv5, computed_cv[5 * TS128_CHAINING_VALUE_BYTE_LEN..6 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv6, computed_cv[6 * TS128_CHAINING_VALUE_BYTE_LEN..7 * TS128_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv7, computed_cv[7 * TS128_CHAINING_VALUE_BYTE_LEN..]);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_8xcompute_chaining_value_with_ts256() {
        if !is_x86_feature_detected!("avx512f") {
            return;
        }

        let mut rng = rand::rng();
        let chunkx8: [u8; CHUNKX8_BYTE_LEN] = rng.random();

        let (chunk0, chunk1, chunk2, chunk3, chunk4, chunk5, chunk6, chunk7) = {
            let (chunk0, rest) = chunkx8.split_at(CHUNK_BYTE_LEN);
            let (chunk1, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk2, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk3, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk4, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk5, rest) = rest.split_at(CHUNK_BYTE_LEN);
            let (chunk6, chunk7) = rest.split_at(CHUNK_BYTE_LEN);

            (chunk0, chunk1, chunk2, chunk3, chunk4, chunk5, chunk6, chunk7)
        };

        let expected_cv0 = compute_cv_with_ts256(chunk0.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv1 = compute_cv_with_ts256(chunk1.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv2 = compute_cv_with_ts256(chunk2.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv3 = compute_cv_with_ts256(chunk3.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv4 = compute_cv_with_ts256(chunk4.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv5 = compute_cv_with_ts256(chunk5.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv6 = compute_cv_with_ts256(chunk6.try_into().expect("must not fail to convert slice to array reference"));
        let expected_cv7 = compute_cv_with_ts256(chunk7.try_into().expect("must not fail to convert slice to array reference"));

        let mut computed_cv = [0u8; 8 * TS256_CHAINING_VALUE_BYTE_LEN];
        unsafe { cvx8::compute_chaining_valuex8::<TS256_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS256_CHAINING_VALUE_BYTE_LEN>(&chunkx8, &mut computed_cv) };

        assert_eq!(expected_cv0, computed_cv[..TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv1, computed_cv[TS256_CHAINING_VALUE_BYTE_LEN..2 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv2, computed_cv[2 * TS256_CHAINING_VALUE_BYTE_LEN..3 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv3, computed_cv[3 * TS256_CHAINING_VALUE_BYTE_LEN..4 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv4, computed_cv[4 * TS256_CHAINING_VALUE_BYTE_LEN..5 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv5, computed_cv[5 * TS256_CHAINING_VALUE_BYTE_LEN..6 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv6, computed_cv[6 * TS256_CHAINING_VALUE_BYTE_LEN..7 * TS256_CHAINING_VALUE_BYTE_LEN]);
        assert_eq!(expected_cv7, computed_cv[7 * TS256_CHAINING_VALUE_BYTE_LEN..]);
    }
}
