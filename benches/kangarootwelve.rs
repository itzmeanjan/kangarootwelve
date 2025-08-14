use kangarootwelve::KangarooTwelve;
use rand::Rng;
use std::{fmt::Debug, time::Duration};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::default().bytes_format(divan::counter::BytesFormat::Binary).main();
}

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

struct K12Config {
    msg_byte_len: usize,
    digest_byte_len: usize,
}

impl Debug for K12Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Hashing {} message, producing {} digest",
            bytes_to_human_readable(self.msg_byte_len),
            bytes_to_human_readable(self.digest_byte_len)
        ))
    }
}

const ARGS: &[K12Config] = &[
    K12Config {
        msg_byte_len: 1usize << 5,
        digest_byte_len: 32,
    },
    K12Config {
        msg_byte_len: 1usize << 6,
        digest_byte_len: 32,
    },
    K12Config {
        msg_byte_len: 1usize << 10,
        digest_byte_len: 32,
    },
    K12Config {
        msg_byte_len: 1usize << 15,
        digest_byte_len: 32,
    },
    K12Config {
        msg_byte_len: 1usize << 20,
        digest_byte_len: 32,
    },
    K12Config {
        msg_byte_len: 1usize << 25,
        digest_byte_len: 32,
    },
    K12Config {
        msg_byte_len: 1usize << 30,
        digest_byte_len: 32,
    },
];

#[divan::bench(args = ARGS, max_time = Duration::from_secs(100), skip_ext_time = true)]
fn k12(bencher: divan::Bencher, k12_config: &K12Config) {
    bencher
        .counter(divan::counter::BytesCount::new(k12_config.msg_byte_len + k12_config.digest_byte_len))
        .with_inputs(|| {
            let mut rng = rand::thread_rng();

            let msg = (0..k12_config.msg_byte_len).map(|_| rng.r#gen()).collect::<Vec<u8>>();
            let dig = vec![0u8; k12_config.digest_byte_len];

            (msg, dig)
        })
        .bench_refs(|(msg, dig)| {
            let mut hasher = KangarooTwelve::hash(divan::black_box(msg), divan::black_box(&[]));
            hasher.squeeze(divan::black_box(dig));
        });
}
