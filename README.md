# kangarootwelve
BlaKE12: Blazing-fast KEccak on 12 rounds

## Overview

KangarooTwelve is a fast and secure arbitrary output-length hash function which performs much better than hash and extendable output functions specified on FIPS 202 ( more @ https://dx.doi.org/10.6028/NIST.FIPS.202 ). KangarooTwelve ( aka K12 or BlaKE12 - see more @ https://blake12.org ) is built on top of round-reduced keccak-p[1600, 12] permutation. To be more specific KangarooTwelve can be implemented on top of TurboSHAKE - which is a family of extendable output functions, recently specified on https://ia.cr/2023/342. The sponge mode of KangarooTwelve uses 256 -bit wide capacity i.e. it can use TurboSHAKE128 as underlying construction. K12 possesses a built-in parallel hashing mode for long ( >=8KB ) messages which can efficiently be exploited by multiple-cores or SIMD instructions. Another important gain of K12 is that its parallel design doesn't impact performance when hashing short ( <8KB ) messages. KangarooTwelve is specified on https://keccak.team/files/KangarooTwelve.pdf.

Here I'm developing/ maintaining a Rust library which implements KangarooTwelve specification s.t. it implements non-incremental absorption API with arbitrary times squeeze support. In coming weeks, I plan to support incremental hashing API i.e. one can construct a K12 hasher object for absorbing message bytes arbitrary many times, then finalize using customization string, before squeezing bytes out of sponge state. See [below](#usage) for example, showing usage of K12 XOF API.

## Prerequisites

Rust stable toolchain; see https://rustup.rs for installation guide. MSRV for this crate is 1.85.0.

```bash
# When developing this library, I was using
$ rustc --version
rustc 1.89.0 (29483883e 2025-08-04)
```

## Testing

For ensuring functional correctness of KangarooTwelve XOF's implementation, I use test vectors from section 4 ( on page 9 ) and Appendix A ( on page 17 ) of https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve. Issue following command to run test cases.

```bash
# Testing on host, first with `default` feature, then with `multi_threaded` feature enabled.
make test

# Testing on web assembly target, using `wasmtime`.
rustup target add wasm32-wasip1
rustup target add wasm32-wasip2
cargo install wasmtime-cli@35.0.0 --locked

make test-wasm
```

## Benchmarking

Issue following command for benchmarking KangarooTwelve extendable output function's (XOF) implementation, for varying input sizes and fixed squeezed output size ( = 32 -bytes ).

```bash
make bench # First runs with `default` feature, then with `multi_threaded` feature
```

> [!WARNING]
> When benchmarking make sure you've disabled CPU frequency scaling, otherwise numbers you see can be misleading. I find https://github.com/google/benchmark/blob/b40db869/docs/reducing_variance.md helpful.


### On 12th Gen Intel(R) Core(TM) i7-1260P

Running benchmarks on `Linux 6.14.0-27-generic x86_64`, compiled with `rustc 1.89.0 (29483883e 2025-08-04)`.

```bash
# With `default` feature - serial absorption of input message

Timer precision: 23 ns
kangarootwelve                                            fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ k12                                                                  │               │               │               │         │
   ├─ Hashing 1.00 GB message, producing 32.00 B digest   873.7 ms      │ 880.6 ms      │ 877.8 ms      │ 877.7 ms      │ 100     │ 100
   │                                                      1.144 GiB/s   │ 1.135 GiB/s   │ 1.139 GiB/s   │ 1.139 GiB/s   │         │
   ├─ Hashing 1.00 KB message, producing 32.00 B digest   1.164 µs      │ 3.577 µs      │ 1.23 µs       │ 1.281 µs      │ 100     │ 100
   │                                                      864.5 MiB/s   │ 281.4 MiB/s   │ 818.1 MiB/s   │ 785.5 MiB/s   │         │
   ├─ Hashing 1.00 MB message, producing 32.00 B digest   778.6 µs      │ 920.9 µs      │ 810.4 µs      │ 816.2 µs      │ 100     │ 100
   │                                                      1.254 GiB/s   │ 1.06 GiB/s    │ 1.205 GiB/s   │ 1.196 GiB/s   │         │
   ├─ Hashing 32.00 B message, producing 32.00 B digest   246.6 ns      │ 334.2 ns      │ 249.5 ns      │ 255.4 ns      │ 100     │ 1600
   │                                                      247.5 MiB/s   │ 182.6 MiB/s   │ 244.5 MiB/s   │ 238.9 MiB/s   │         │
   ├─ Hashing 32.00 KB message, producing 32.00 B digest  24.47 µs      │ 28.57 µs      │ 25.25 µs      │ 25.32 µs      │ 100     │ 100
   │                                                      1.248 GiB/s   │ 1.068 GiB/s   │ 1.209 GiB/s   │ 1.206 GiB/s   │         │
   ├─ Hashing 32.00 MB message, producing 32.00 B digest  26.77 ms      │ 27.79 ms      │ 27.17 ms      │ 27.21 ms      │ 100     │ 100
   │                                                      1.167 GiB/s   │ 1.124 GiB/s   │ 1.149 GiB/s   │ 1.148 GiB/s   │         │
   ╰─ Hashing 64.00 B message, producing 32.00 B digest   258.9 ns      │ 446.8 ns      │ 276.6 ns      │ 284 ns        │ 100     │ 800
                                                          353.5 MiB/s   │ 204.8 MiB/s   │ 330.9 MiB/s   │ 322.3 MiB/s   │         │

# With `multi_threaded` feature - Using `rayon` for data-parallel absorption of input message

Timer precision: 23 ns
kangarootwelve                                            fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ k12                                                                  │               │               │               │         │
   ├─ Hashing 1.00 GB message, producing 32.00 B digest   123.6 ms      │ 154.2 ms      │ 131.4 ms      │ 132.2 ms      │ 100     │ 100
   │                                                      8.086 GiB/s   │ 6.482 GiB/s   │ 7.604 GiB/s   │ 7.559 GiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        167         │ 167           │ 167           │ 167           │         │
   │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        184         │ 168           │ 168           │ 168.1         │         │
   │                                                        72.49 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.4 KiB      │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        20          │ 4             │ 4             │ 4.16          │         │
   │                                                        4.018 MiB   │ 4 MiB         │ 4 MiB         │ 4.001 MiB     │         │
   │                                                      grow:         │               │               │               │         │
   │                                                        3           │ 0             │ 0             │ 0.03          │         │
   │                                                        288 B       │ 0 B           │ 0 B           │ 2.88 B        │         │
   ├─ Hashing 1.00 KB message, producing 32.00 B digest   1.235 µs      │ 4.408 µs      │ 1.268 µs      │ 1.333 µs      │ 100     │ 100
   │                                                      814.8 MiB/s   │ 228.4 MiB/s   │ 793.6 MiB/s   │ 755.4 MiB/s   │         │
   ├─ Hashing 1.00 MB message, producing 32.00 B digest   414.2 µs      │ 1.152 ms      │ 585.8 µs      │ 607.3 µs      │ 100     │ 100
   │                                                      2.357 GiB/s   │ 867.4 MiB/s   │ 1.667 GiB/s   │ 1.607 GiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        167         │ 167           │ 167           │ 167           │         │
   │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        168         │ 168           │ 168           │ 168           │         │
   │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 4             │         │
   │                                                        5 KiB       │ 5 KiB         │ 5 KiB         │ 5 KiB         │         │
   ├─ Hashing 32.00 B message, producing 32.00 B digest   257.9 ns      │ 603.8 ns      │ 262.9 ns      │ 305.1 ns      │ 100     │ 800
   │                                                      236.5 MiB/s   │ 101 MiB/s     │ 232.1 MiB/s   │ 200 MiB/s     │         │
   ├─ Hashing 32.00 KB message, producing 32.00 B digest  80.11 µs      │ 236.7 µs      │ 123.5 µs      │ 126.1 µs      │ 100     │ 100
   │                                                      390.4 MiB/s   │ 132.1 MiB/s   │ 253.1 MiB/s   │ 247.9 MiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        47          │ 47            │ 47            │ 47            │         │
   │                                                        15.32 KiB   │ 15.32 KiB     │ 15.32 KiB     │ 15.32 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        48          │ 48            │ 48            │ 48            │         │
   │                                                        15.39 KiB   │ 15.39 KiB     │ 15.39 KiB     │ 15.39 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 4             │         │
   │                                                        384 B       │ 384 B         │ 384 B         │ 384 B         │         │
   ├─ Hashing 32.00 MB message, producing 32.00 B digest  3.809 ms      │ 5.732 ms      │ 4.655 ms      │ 4.621 ms      │ 100     │ 100
   │                                                      8.202 GiB/s   │ 5.45 GiB/s    │ 6.712 GiB/s   │ 6.761 GiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        167         │ 167           │ 167           │ 167           │         │
   │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        168         │ 168           │ 168           │ 168           │         │
   │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 4             │         │
   │                                                        129 KiB     │ 129 KiB       │ 129 KiB       │ 129 KiB       │         │
   ╰─ Hashing 64.00 B message, producing 32.00 B digest   359.8 ns      │ 2.395 µs      │ 389.8 ns      │ 432.3 ns      │ 100     │ 100
                                                          254.4 MiB/s   │ 38.21 MiB/s   │ 234.8 MiB/s   │ 211.7 MiB/s   │         │
```

## Usage

Getting started with using KangarooTwelve extendable output function API is pretty easy

1) Add `kangarootwelve` as project dependency in your `Cargo.toml` file

```toml
[dependencies]
# either
kangarootwelve = { git = "https://github.com/itzmeanjan/kangarootwelve" }
# or
kangarootwelve = "0.1.1"
# or if interested in using multi-threaded K12 absorption for long messages
kangarootwelve = { version = "0.1.1", features = "multi_threaded" }
```

2) Right now KangarooTwelve offers only non-incremental absorption API, so absorb message and customization string into sponge state using `hash()` function, which returns an XOF object, holding sponge in its finalized state.

```rust
use kangarootwelve::KangarooTwelve;
use rand::{thread_rng, RngCore};

fn main() {
  const MLEN: usize = 64;
  const CSTRLEN: usize = 1;
  const DLEN: usize = 32;

  let mut msg = vec![0u8; MLEN];
  let mut cstr = vec![0u8; CSTRLEN]; // you can keep it empty
  let mut dig = vec![0u8; DLEN];

  let mut rng = thread_rng();
  rng.fill_bytes(&mut msg);
  cstr[0] = 0xff;

  let mut hasher = KangarooTwelve::hash(&msg, &cstr);
  // ...
}
```

3) Sponge is ready to be squeezed i.e. now you can use returned XOF object for squeezing arbitrary number of bytes arbitrary number of times.

```rust
hasher.squeeze(&mut dig[..DLEN / 2]);
hasher.squeeze(&mut dig[DLEN / 2..]);
```

I maintain following example, demonstrating usage of KangarooTwelve eXtendable Output Function (XOF).

- [k12](./examples/k12.rs)

```
cargo run --release --example k12

Message              = 03959c2ffc95ac27dbf150fa1bbd4eebeaf531cf5bfd93680a197453350260ca86d78ba9376c8bf55350a7b695f473c486853d955de5eef456a7bc14d22316c5
Customization String = ff
Digest               = 1ab580fbc34d1e49d4c6b1b34b8e9d6b25e0ee60185559e3c7384e5c15629781
```
