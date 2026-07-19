use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kangarootwelve::{KT128, KT256};
use rand::Rng;
use std::{hint::black_box, time::Duration};

#[cfg(feature = "cuda")]
use kangarootwelve::DeviceBuffer;
#[cfg(feature = "cuda")]
use std::time::Instant;

fn bytes_to_human_readable(bytes: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut bytes = bytes as f64;
    let mut unit_index = 0;

    while bytes >= 1024.0 && unit_index < units.len() - 1 {
        bytes /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", bytes, units[unit_index])
}

struct KangarooTwelveConfig {
    msg_byte_len: usize,
    digest_byte_len: usize,
}

impl KangarooTwelveConfig {
    fn label(&self) -> String {
        format!(
            "hashing {} message, producing {} digest",
            bytes_to_human_readable(self.msg_byte_len),
            bytes_to_human_readable(self.digest_byte_len)
        )
    }
}

const ARGS: &[KangarooTwelveConfig] = &[
    KangarooTwelveConfig {
        msg_byte_len: 1usize << 10,
        digest_byte_len: 32,
    },
    KangarooTwelveConfig {
        msg_byte_len: 1usize << 20,
        digest_byte_len: 32,
    },
    KangarooTwelveConfig {
        msg_byte_len: 1usize << 30,
        digest_byte_len: 32,
    },
    KangarooTwelveConfig {
        msg_byte_len: 1usize << 32,
        digest_byte_len: 32,
    },
    #[cfg(feature = "cuda")]
    KangarooTwelveConfig {
        msg_byte_len: 1usize << 34,
        digest_byte_len: 32,
    },
    #[cfg(feature = "cuda")]
    KangarooTwelveConfig {
        msg_byte_len: 1usize << 35,
        digest_byte_len: 32,
    },
];

fn kt128(c: &mut Criterion) {
    let mut rng = rand::rng();
    let mut group = c.benchmark_group("kt128");

    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    for cfg in ARGS {
        let msg = (0..cfg.msg_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

        group.throughput(Throughput::Bytes((cfg.msg_byte_len + cfg.digest_byte_len) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(cfg.label()), &msg, |b, msg| {
            #[cfg(feature = "cuda")]
            b.iter_custom(|iters| {
                let dmsg = DeviceBuffer::new(msg).unwrap();
                let mut digest = vec![0u8; cfg.digest_byte_len];
                let mut total = Duration::ZERO;

                for _ in 0..iters {
                    let (mut xof, hashed_in) = KT128::hash_device(black_box(&dmsg), black_box(&[]));

                    let squeeze_begin = Instant::now();
                    xof.squeeze(black_box(&mut digest));

                    total += hashed_in + squeeze_begin.elapsed();
                }

                total
            });

            #[cfg(not(feature = "cuda"))]
            b.iter_batched(
                || vec![0u8; cfg.digest_byte_len],
                |mut digest| {
                    KT128::hash(black_box(msg), black_box(&[])).squeeze(black_box(&mut digest));
                    digest
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn kt256(c: &mut Criterion) {
    let mut rng = rand::rng();
    let mut group = c.benchmark_group("kt256");

    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    for cfg in ARGS {
        let msg = (0..cfg.msg_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

        group.throughput(Throughput::Bytes((cfg.msg_byte_len + cfg.digest_byte_len) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(cfg.label()), &msg, |b, msg| {
            #[cfg(feature = "cuda")]
            b.iter_custom(|iters| {
                let dmsg = DeviceBuffer::new(msg).unwrap();
                let mut digest = vec![0u8; cfg.digest_byte_len];
                let mut total = Duration::ZERO;

                for _ in 0..iters {
                    let (mut xof, hashed_in) = KT256::hash_device(black_box(&dmsg), black_box(&[]));

                    let squeeze_begin = Instant::now();
                    xof.squeeze(black_box(&mut digest));

                    total += hashed_in + squeeze_begin.elapsed();
                }

                total
            });

            #[cfg(not(feature = "cuda"))]
            b.iter_batched(
                || vec![0u8; cfg.digest_byte_len],
                |mut digest| {
                    KT256::hash(black_box(msg), black_box(&[])).squeeze(black_box(&mut digest));
                    digest
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, kt128, kt256);
criterion_main!(benches);
