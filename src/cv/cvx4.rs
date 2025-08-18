use crate::cv::consts::CHUNK_BYTE_LEN;
use crate::keccak::keccakx4;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[allow(unused_unsafe)]
pub fn compute_chaining_valuex4<const NUM_RATE_BITS: usize, const DOMAIN_SEPARATOR: u8, const CV_SIZE: usize>(
    chunk0: &[u8; CHUNK_BYTE_LEN],
    chunk1: &[u8; CHUNK_BYTE_LEN],
    chunk2: &[u8; CHUNK_BYTE_LEN],
    chunk3: &[u8; CHUNK_BYTE_LEN],
) -> ([u8; CV_SIZE], [u8; CV_SIZE], [u8; CV_SIZE], [u8; CV_SIZE]) {
    unsafe {
        let num_rate_bytes = NUM_RATE_BITS / u8::BITS as usize;
        let num_rate_words = NUM_RATE_BITS / turboshake::keccak::W;

        let num_bytes_in_last_block = CHUNK_BYTE_LEN % num_rate_bytes;
        let num_words_in_last_block = num_bytes_in_last_block / u8::BITS as usize;

        let mut keccak_statex4 = [_mm256_setzero_si256(); turboshake::keccak::LANE_CNT];

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

        let mut cv0 = [0u8; CV_SIZE];
        let mut cv1 = [0u8; CV_SIZE];
        let mut cv2 = [0u8; CV_SIZE];
        let mut cv3 = [0u8; CV_SIZE];

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

        (cv0, cv1, cv2, cv3)
    }
}
