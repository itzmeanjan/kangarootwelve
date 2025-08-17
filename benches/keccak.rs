use rand::Rng;
use std::time::Duration;

fn main() {
    divan::Divan::default().bytes_format(divan::counter::BytesFormat::Binary).main();
}

#[divan::bench(name="12-rounds 1x SIMD parallel keccak (u64)", min_time = Duration::from_secs(10), max_time = Duration::from_secs(100), skip_ext_time = true)]
fn keccak(bencher: divan::Bencher) {
    let mut rng = rand::rng();
    let mut keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();

    bencher
        .counter(divan::counter::BytesCount::new(200usize))
        .bench_local(|| turboshake::keccak::permute(divan::black_box(&mut keccak_state)));
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[divan::bench(name="12-rounds 2x SIMD parallel keccak (sse2)", min_time = Duration::from_secs(10), max_time = Duration::from_secs(100), skip_ext_time = true)]
fn keccakx2(bencher: divan::Bencher) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::{__m128i, _mm_set1_epi64x};

    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::{__m128i, _mm_set1_epi64x};

    use kangarootwelve::keccak;

    if !is_x86_feature_detected!("sse2") {
        return;
    }

    let mut rng = rand::rng();

    let keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();
    let mut keccak_statex2: [__m128i; turboshake::keccak::LANE_CNT] = keccak_state
        .iter()
        .map(|&lane| unsafe { _mm_set1_epi64x(lane as i64) })
        .collect::<Vec<__m128i>>()
        .try_into()
        .expect("Must be able to form 2x keccak-p[1600], backed by SSE2 registers");

    bencher
        .counter(divan::counter::BytesCount::new(400usize))
        .bench_local(|| unsafe { keccak::keccakx2::permute(divan::black_box(&mut keccak_statex2)) });
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[divan::bench(name="12-rounds 4x SIMD parallel keccak (avx2)", min_time = Duration::from_secs(10), max_time = Duration::from_secs(100), skip_ext_time = true)]
fn keccakx4(bencher: divan::Bencher) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::{__m256i, _mm256_set1_epi64x};

    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::{__m256i, _mm256_set1_epi64x};

    use kangarootwelve::keccak;

    if !is_x86_feature_detected!("avx2") {
        return;
    }

    let mut rng = rand::rng();

    let keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();
    let mut keccak_statex4: [__m256i; turboshake::keccak::LANE_CNT] = keccak_state
        .iter()
        .map(|&lane| unsafe { _mm256_set1_epi64x(lane as i64) })
        .collect::<Vec<__m256i>>()
        .try_into()
        .expect("Must be able to form 4x keccak-p[1600], backed by AVX2 registers");

    bencher
        .counter(divan::counter::BytesCount::new(800usize))
        .bench_local(|| unsafe { keccak::keccakx4::permute(divan::black_box(&mut keccak_statex4)) });
}
