use rand::Rng;
use std::time::Duration;

fn main() {
    divan::Divan::default().bytes_format(divan::counter::BytesFormat::Binary).main();
}

#[divan::bench(name="12-rounds keccak", max_time = Duration::from_secs(100), skip_ext_time = true)]
fn keccak(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let mut rng = rand::rng();
            let keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();

            keccak_state
        })
        .input_counter(|keccak_value| divan::counter::BytesCount::new(keccak_value.len() * std::mem::size_of_val(&keccak_value[0])))
        .bench_refs(|keccak_state| turboshake::keccak::permute(divan::black_box(keccak_state)));
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[divan::bench(name="12-rounds keccak x2", max_time = Duration::from_secs(100), skip_ext_time = true)]
fn keccakx2(bencher: divan::Bencher) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::{__m128i, _mm_set1_epi64x};

    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::{__m128i, _mm_set1_epi64x};

    use kangarootwelve::keccak;

    if !is_x86_feature_detected!("sse2") {
        return;
    }

    bencher
        .with_inputs(|| {
            let mut rng = rand::rng();

            let keccak_state: [u64; turboshake::keccak::LANE_CNT] = rng.random();
            let keccak_statex2: [__m128i; turboshake::keccak::LANE_CNT] = keccak_state
                .iter()
                .map(|&lane| unsafe { _mm_set1_epi64x(lane as i64) })
                .collect::<Vec<__m128i>>()
                .try_into()
                .expect("Must be able to form 2x keccak-p[1600], backed by SSE2 registers");

            keccak_statex2
        })
        .input_counter(|keccak_value| divan::counter::BytesCount::new(keccak_value.len() * std::mem::size_of_val(&keccak_value[0])))
        .bench_refs(|keccak_statex2| unsafe { keccak::keccakx2::permute(divan::black_box(keccak_statex2)) });
}
