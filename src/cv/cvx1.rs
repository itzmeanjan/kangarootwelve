use turboshake::{keccak, sponge};

pub fn compute_chaining_value<const NUM_RATE_BYTES: usize, const DOMAIN_SEPARATOR: u8, const CV_SIZE: usize>(chunk: &[u8], chaining_value: &mut [u8; CV_SIZE]) {
    let mut state = [0u64; keccak::LANE_CNT];
    let mut offset = 0;

    sponge::absorb::<NUM_RATE_BYTES>(&mut state, &mut offset, chunk);
    sponge::finalize::<NUM_RATE_BYTES, DOMAIN_SEPARATOR>(&mut state, &mut offset);

    offset = NUM_RATE_BYTES;
    sponge::squeeze::<NUM_RATE_BYTES>(&mut state, &mut offset, chaining_value);
}

#[cfg(test)]
mod test {
    use crate::cv::{consts::CHUNK_BYTE_LEN, cvx1::compute_chaining_value};
    use rand::Rng;
    use turboshake::{TurboShake128, TurboShake256};

    const TS128_NUM_RATE_BITS: usize = 1600 - 256;
    const TS256_NUM_RATE_BITS: usize = 1600 - 512;

    const TS128_NUM_RATE_BYTES: usize = TS128_NUM_RATE_BITS / 8;
    const TS256_NUM_RATE_BYTES: usize = TS256_NUM_RATE_BITS / 8;

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

    #[test]
    fn test_compute_chaining_value_with_ts128() {
        let mut rng = rand::rng();
        let chunk: [u8; CHUNK_BYTE_LEN] = rng.random();

        let expected_cv = compute_cv_with_ts128(&chunk);

        let mut computed_cv = [0u8; TS128_CHAINING_VALUE_BYTE_LEN];
        compute_chaining_value::<TS128_NUM_RATE_BYTES, DOMAIN_SEPARATOR, TS128_CHAINING_VALUE_BYTE_LEN>(&chunk, &mut computed_cv);

        assert_eq!(expected_cv, computed_cv);
    }

    #[test]
    fn test_compute_chaining_value_with_ts256() {
        let mut rng = rand::rng();
        let chunk: [u8; CHUNK_BYTE_LEN] = rng.random();

        let expected_cv = compute_cv_with_ts256(&chunk);

        let mut computed_cv = [0u8; TS256_CHAINING_VALUE_BYTE_LEN];
        compute_chaining_value::<TS256_NUM_RATE_BYTES, DOMAIN_SEPARATOR, TS256_CHAINING_VALUE_BYTE_LEN>(&chunk, &mut computed_cv);

        assert_eq!(expected_cv, computed_cv);
    }
}
