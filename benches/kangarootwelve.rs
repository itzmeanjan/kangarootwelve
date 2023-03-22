use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use kangarootwelve::KangarooTwelve;
use rand::{thread_rng, RngCore};

fn k12<const MLEN: usize, const CSTRLEN: usize, const DLEN: usize>(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("K12");
    group.throughput(Throughput::Bytes((MLEN + CSTRLEN) as u64));

    group.bench_function(&format!("{}/{} (cached)", MLEN, DLEN), |bench| {
        let mut msg = vec![0u8; MLEN];
        let mut cstr = vec![0u8; CSTRLEN];
        let mut dig = vec![0u8; DLEN];

        rng.fill_bytes(&mut msg);
        rng.fill_bytes(&mut cstr);

        bench.iter(|| {
            let mut hasher = KangarooTwelve::hash(black_box(&msg), black_box(&cstr));
            hasher.squeeze(black_box(&mut dig));
        });
    });
    group.bench_function(&format!("{}/{} (random)", MLEN, DLEN), |bench| {
        let mut msg = vec![0u8; MLEN];
        let mut cstr = vec![0u8; CSTRLEN];
        let mut dig = vec![0u8; DLEN];

        rng.fill_bytes(&mut msg);
        rng.fill_bytes(&mut cstr);

        bench.iter_batched(
            || (msg.clone(), cstr.clone()),
            |(msg, cstr)| {
                let mut hasher = KangarooTwelve::hash(black_box(&msg), black_box(&cstr));
                hasher.squeeze(black_box(&mut dig));
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(kangarootwelve, k12<32, 0, 32>, k12<64, 0, 32>, k12<128, 0, 32>, k12<256, 0, 32>, k12<512, 0, 32>, k12<{1*(1 << 10)}, 0, 32>, k12<{2*(1 << 10)}, 0, 32>, k12<{4*(1 << 10)}, 0, 32>, k12<{8*(1 << 10)}, 0, 32>, k12<{1*(1 << 20)}, 0, 32>, k12<{2*(1 << 20)}, 0, 32>, k12<{4*(1 << 20)}, 0, 32>, k12<{8*(1 << 20)}, 0, 32>);
criterion_main!(kangarootwelve);
