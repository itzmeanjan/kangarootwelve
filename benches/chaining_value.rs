use kangarootwelve::cv;
use rand::Rng;
use std::time::Duration;
use turboshake::TurboShake128;

fn main() {
    divan::Divan::default().bytes_format(divan::counter::BytesFormat::Binary).main();
}

const DOMAIN_SEPARATOR: u8 = 0x0b;
const TS128_NUM_RATE_BITS: usize = 1600 - 256;
const TS128_CHAINING_VALUE_BYTE_LEN: usize = 32;

#[divan::bench(name="1x SIMD parallel compute chaining value using TurboSHAKE128 (u64)", min_time = Duration::from_secs(10), max_time = Duration::from_secs(100), skip_ext_time = true)]
fn compute_chaining_value(bencher: divan::Bencher) {
    let mut rng = rand::rng();
    let chunk: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();

    bencher.counter(divan::counter::BytesCount::new(chunk.len())).bench_local(|| {
        let mut cv = [0u8; TS128_CHAINING_VALUE_BYTE_LEN];
        let mut ts128 = TurboShake128::default();

        let _ = ts128.absorb(divan::black_box(&chunk));
        let _ = ts128.finalize::<DOMAIN_SEPARATOR>();
        let _ = ts128.squeeze(divan::black_box(&mut cv));

        cv
    });
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[divan::bench(name="2x SIMD parallel compute chaining value using TurboSHAKE128 (sse2)", min_time = Duration::from_secs(10), max_time = Duration::from_secs(100), skip_ext_time = true)]
fn compute_chaining_valuex2(bencher: divan::Bencher) {
    if !is_x86_feature_detected!("sse2") {
        return;
    }

    let mut rng = rand::rng();

    let chunk0: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();
    let chunk1: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();

    bencher
        .counter(divan::counter::BytesCount::new(chunk0.len() + chunk1.len()))
        .bench_local(|| unsafe {
            cv::cvx2::compute_chaining_valuex2::<TS128_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS128_CHAINING_VALUE_BYTE_LEN>(
                divan::black_box(&chunk0),
                divan::black_box(&chunk1),
            )
        });
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[divan::bench(name="4x SIMD parallel compute chaining value using TurboSHAKE128 (avx2)", min_time = Duration::from_secs(10), max_time = Duration::from_secs(100), skip_ext_time = true)]
fn compute_chaining_valuex4(bencher: divan::Bencher) {
    if !is_x86_feature_detected!("avx2") {
        return;
    }

    let mut rng = rand::rng();

    let chunk0: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();
    let chunk1: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();
    let chunk2: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();
    let chunk3: [u8; cv::consts::CHUNK_BYTE_LEN] = rng.random();

    bencher
        .counter(divan::counter::BytesCount::new(chunk0.len() + chunk1.len() + chunk2.len() + chunk3.len()))
        .bench_local(|| unsafe {
            cv::cvx4::compute_chaining_valuex4::<TS128_NUM_RATE_BITS, DOMAIN_SEPARATOR, TS128_CHAINING_VALUE_BYTE_LEN>(
                divan::black_box(&chunk0),
                divan::black_box(&chunk1),
                divan::black_box(&chunk2),
                divan::black_box(&chunk3),
            )
        });
}
