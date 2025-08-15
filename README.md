# kangarootwelve
BlaKE12: Blazing-fast KEccak on 12 rounds

## Overview

KangarooTwelve is a family of fast and secure arbitrary output-length hash functions which performs much better than hash and extendable output functions specified on FIPS 202 ( more @ https://dx.doi.org/10.6028/NIST.FIPS.202 ). KangarooTwelve ( aka K12 or BlaKE12 - see more @ https://blake12.org ) is built on top of round-reduced keccak-p[1600, 12] permutation. To be more specific KangarooTwelve can be implemented on top of TurboSHAKE - which is a family of extendable output functions, recently specified on https://ia.cr/2023/342. KangarooTwelve offers two instances

- KT128, uses TurboSHAKE128 as underlying chunk hasher.
- KT256, uses TurboSHAKE256 as underlying chunk hasher.

K12 possesses a built-in parallel tree hashing mode (using SAKURA coding) for long ( >=8KB ) messages which can efficiently be exploited by multiple-cores or SIMD instructions. Another important gain of K12 is that its parallel design doesn't impact performance when hashing short ( <8KB ) messages. Originally KangarooTwelve (which is now renamed to KT128) was specified on https://keccak.team/files/KangarooTwelve.pdf. The latest specification, defining both KT128 and KT256 can be found @ https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve.

Here I'm developing/ maintaining a Rust library which implements KangarooTwelve specification s.t. it implements non-incremental absorption API with arbitrary times squeeze support. In coming weeks, I plan to support incremental hashing API i.e. one can construct a K12 hasher object for absorbing message bytes arbitrary many times, then finalize using customization string, before squeezing bytes out of sponge state. See [below](#usage) for example, showing usage of K12 XOF API.

## Prerequisites

Rust stable toolchain; see https://rustup.rs for installation guide. MSRV for this crate is 1.85.0.

```bash
# When developing this library, I was using
$ rustc --version
rustc 1.89.0 (29483883e 2025-08-04)
```

## Testing

For ensuring functional correctness of KangarooTwelve family of XOF implementation, I use test vectors from section 5 (on page 12) of https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve. Issue following command to run all tests.

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

Issue following command for benchmarking KangarooTwelve family of extendable output functions (XOF), for varying input sizes.

```bash
make bench # First runs with `default` feature, then with `multi_threaded` feature
```

> [!WARNING]
> When benchmarking make sure you've disabled CPU frequency scaling, otherwise numbers you see can be misleading. I find https://github.com/google/benchmark/blob/b40db869/docs/reducing_variance.md helpful.


### On 12th Gen Intel(R) Core(TM) i7-1260P

Running benchmarks on `Linux 6.14.0-27-generic x86_64`, compiled with `rustc 1.89.0 (29483883e 2025-08-04)`.

```bash
# With `default` feature - serial absorption of input message

Timer precision: 22 ns
kangarootwelve                                            fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ kt128                                                                │               │               │               │         │
│  ├─ Hashing 1.00 GB message, producing 32.00 B digest   873.3 ms      │ 880.3 ms      │ 876.6 ms      │ 876.5 ms      │ 100     │ 100
│  │                                                      1.145 GiB/s   │ 1.135 GiB/s   │ 1.14 GiB/s    │ 1.14 GiB/s    │         │
│  ├─ Hashing 1.00 KB message, producing 32.00 B digest   1.157 µs      │ 3.5 µs        │ 1.233 µs      │ 1.281 µs      │ 100     │ 100
│  │                                                      869.7 MiB/s   │ 287.6 MiB/s   │ 816.1 MiB/s   │ 785.6 MiB/s   │         │
│  ├─ Hashing 1.00 MB message, producing 32.00 B digest   778.8 µs      │ 943.4 µs      │ 837.2 µs      │ 833.2 µs      │ 100     │ 100
│  │                                                      1.253 GiB/s   │ 1.035 GiB/s   │ 1.166 GiB/s   │ 1.172 GiB/s   │         │
│  ├─ Hashing 32.00 B message, producing 32.00 B digest   255.6 ns      │ 413.6 ns      │ 305.6 ns      │ 299.6 ns      │ 100     │ 800
│  │                                                      238.7 MiB/s   │ 147.5 MiB/s   │ 199.7 MiB/s   │ 203.7 MiB/s   │         │
│  ├─ Hashing 32.00 KB message, producing 32.00 B digest  24.85 µs      │ 30.99 µs      │ 25.49 µs      │ 26.18 µs      │ 100     │ 100
│  │                                                      1.228 GiB/s   │ 1009 MiB/s    │ 1.198 GiB/s   │ 1.166 GiB/s   │         │
│  ├─ Hashing 32.00 MB message, producing 32.00 B digest  26.65 ms      │ 27.89 ms      │ 27.23 ms      │ 27.22 ms      │ 100     │ 100
│  │                                                      1.172 GiB/s   │ 1.12 GiB/s    │ 1.147 GiB/s   │ 1.147 GiB/s   │         │
│  ╰─ Hashing 64.00 B message, producing 32.00 B digest   263.9 ns      │ 352.8 ns      │ 339.5 ns      │ 320.9 ns      │ 100     │ 800
│                                                         346.8 MiB/s   │ 259.4 MiB/s   │ 269.6 MiB/s   │ 285.2 MiB/s   │         │
╰─ kt256                                                                │               │               │               │         │
   ├─ Hashing 1.00 GB message, producing 32.00 B digest   1.093 s       │ 1.101 s       │ 1.096 s       │ 1.096 s       │ 92      │ 92
   │                                                      936.6 MiB/s   │ 929.3 MiB/s   │ 934 MiB/s     │ 933.7 MiB/s   │         │
   ├─ Hashing 1.00 KB message, producing 32.00 B digest   1.271 µs      │ 3.598 µs      │ 1.349 µs      │ 1.362 µs      │ 100     │ 100
   │                                                      791.8 MiB/s   │ 279.8 MiB/s   │ 746 MiB/s     │ 739 MiB/s     │         │
   ├─ Hashing 1.00 MB message, producing 32.00 B digest   990.7 µs      │ 1.152 ms      │ 1.014 ms      │ 1.026 ms      │ 100     │ 100
   │                                                      1009 MiB/s    │ 867.7 MiB/s   │ 985.6 MiB/s   │ 974.6 MiB/s   │         │
   ├─ Hashing 32.00 B message, producing 32.00 B digest   253.2 ns      │ 411.3 ns      │ 257.1 ns      │ 277.2 ns      │ 100     │ 800
   │                                                      241 MiB/s     │ 148.3 MiB/s   │ 237.3 MiB/s   │ 220.1 MiB/s   │         │
   ├─ Hashing 32.00 KB message, producing 32.00 B digest  31.16 µs      │ 36.36 µs      │ 31.93 µs      │ 32.44 µs      │ 100     │ 100
   │                                                      1003 MiB/s    │ 860 MiB/s     │ 979.3 MiB/s   │ 964.1 MiB/s   │         │
   ├─ Hashing 32.00 MB message, producing 32.00 B digest  33.58 ms      │ 34.88 ms      │ 34.19 ms      │ 34.22 ms      │ 100     │ 100
   │                                                      952.8 MiB/s   │ 917.3 MiB/s   │ 935.6 MiB/s   │ 934.8 MiB/s   │         │
   ╰─ Hashing 64.00 B message, producing 32.00 B digest   265.3 ns      │ 426.6 ns      │ 326.9 ns      │ 309.3 ns      │ 100     │ 800
                                                          345 MiB/s     │ 214.6 MiB/s   │ 279.9 MiB/s   │ 295.9 MiB/s   │         │

# With `multi_threaded` feature - Using `rayon` for data-parallel absorption of input message

Timer precision: 22 ns
kangarootwelve                                            fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ kt128                                                                │               │               │               │         │
│  ├─ Hashing 1.00 GB message, producing 32.00 B digest   122 ms        │ 138.2 ms      │ 131.9 ms      │ 132.3 ms      │ 100     │ 100
│  │                                                      8.196 GiB/s   │ 7.232 GiB/s   │ 7.581 GiB/s   │ 7.558 GiB/s   │         │
│  │                                                      max alloc:    │               │               │               │         │
│  │                                                        167         │ 167           │ 167           │ 167           │         │
│  │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
│  │                                                      alloc:        │               │               │               │         │
│  │                                                        184         │ 168           │ 168           │ 168.1         │         │
│  │                                                        72.49 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.4 KiB      │         │
│  │                                                      dealloc:      │               │               │               │         │
│  │                                                        20          │ 4             │ 4             │ 4.16          │         │
│  │                                                        4.018 MiB   │ 4 MiB         │ 4 MiB         │ 4.001 MiB     │         │
│  │                                                      grow:         │               │               │               │         │
│  │                                                        3           │ 0             │ 0             │ 0.03          │         │
│  │                                                        288 B       │ 0 B           │ 0 B           │ 2.88 B        │         │
│  ├─ Hashing 1.00 KB message, producing 32.00 B digest   932.8 ns      │ 3.617 µs      │ 976.8 ns      │ 1.083 µs      │ 100     │ 100
│  │                                                      1.054 GiB/s   │ 278.3 MiB/s   │ 1.006 GiB/s   │ 929.1 MiB/s   │         │
│  ├─ Hashing 1.00 MB message, producing 32.00 B digest   402.4 µs      │ 1.519 ms      │ 574.4 µs      │ 602.2 µs      │ 100     │ 100
│  │                                                      2.426 GiB/s   │ 658.1 MiB/s   │ 1.7 GiB/s     │ 1.621 GiB/s   │         │
│  │                                                      max alloc:    │               │               │               │         │
│  │                                                        167         │ 167           │ 167           │ 167           │         │
│  │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
│  │                                                      alloc:        │               │               │               │         │
│  │                                                        168         │ 168           │ 168           │ 168           │         │
│  │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
│  │                                                      dealloc:      │               │               │               │         │
│  │                                                        4           │ 4             │ 4             │ 4             │         │
│  │                                                        5 KiB       │ 5 KiB         │ 5 KiB         │ 5 KiB         │         │
│  ├─ Hashing 32.00 B message, producing 32.00 B digest   242.8 ns      │ 1.164 µs      │ 249.8 ns      │ 281.9 ns      │ 100     │ 200
│  │                                                      251.3 MiB/s   │ 52.39 MiB/s   │ 244.2 MiB/s   │ 216.5 MiB/s   │         │
│  ├─ Hashing 32.00 KB message, producing 32.00 B digest  73.71 µs      │ 276.3 µs      │ 118 µs        │ 114.8 µs      │ 100     │ 100
│  │                                                      424.3 MiB/s   │ 113.1 MiB/s   │ 264.9 MiB/s   │ 272.3 MiB/s   │         │
│  │                                                      max alloc:    │               │               │               │         │
│  │                                                        47          │ 47            │ 47            │ 47            │         │
│  │                                                        15.32 KiB   │ 15.32 KiB     │ 15.32 KiB     │ 15.32 KiB     │         │
│  │                                                      alloc:        │               │               │               │         │
│  │                                                        48          │ 48            │ 48            │ 48            │         │
│  │                                                        15.39 KiB   │ 15.39 KiB     │ 15.39 KiB     │ 15.39 KiB     │         │
│  │                                                      dealloc:      │               │               │               │         │
│  │                                                        4           │ 4             │ 4             │ 4             │         │
│  │                                                        384 B       │ 384 B         │ 384 B         │ 384 B         │         │
│  ├─ Hashing 32.00 MB message, producing 32.00 B digest  3.803 ms      │ 5.816 ms      │ 4.663 ms      │ 4.641 ms      │ 100     │ 100
│  │                                                      8.216 GiB/s   │ 5.373 GiB/s   │ 6.7 GiB/s     │ 6.733 GiB/s   │         │
│  │                                                      max alloc:    │               │               │               │         │
│  │                                                        167         │ 167           │ 167           │ 167           │         │
│  │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
│  │                                                      alloc:        │               │               │               │         │
│  │                                                        168         │ 168           │ 168           │ 168           │         │
│  │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
│  │                                                      dealloc:      │               │               │               │         │
│  │                                                        4           │ 4             │ 4             │ 4             │         │
│  │                                                        129 KiB     │ 129 KiB       │ 129 KiB       │ 129 KiB       │         │
│  ╰─ Hashing 64.00 B message, producing 32.00 B digest   356.8 ns      │ 26.03 µs      │ 365.8 ns      │ 654.7 ns      │ 100     │ 100
│                                                         256.5 MiB/s   │ 3.516 MiB/s   │ 250.2 MiB/s   │ 139.8 MiB/s   │         │
╰─ kt256                                                                │               │               │               │         │
   ├─ Hashing 1.00 GB message, producing 32.00 B digest   155.6 ms      │ 175.2 ms      │ 163.6 ms      │ 164.8 ms      │ 100     │ 100
   │                                                      6.425 GiB/s   │ 5.705 GiB/s   │ 6.109 GiB/s   │ 6.066 GiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        167         │ 167           │ 167           │ 167           │         │
   │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        168         │ 168           │ 168           │ 168           │         │
   │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 4             │         │
   │                                                        8 MiB       │ 8 MiB         │ 8 MiB         │ 8 MiB         │         │
   ├─ Hashing 1.00 KB message, producing 32.00 B digest   1.065 µs      │ 2.95 µs       │ 1.081 µs      │ 1.162 µs      │ 100     │ 100
   │                                                      944.8 MiB/s   │ 341.2 MiB/s   │ 930.8 MiB/s   │ 866.2 MiB/s   │         │
   ├─ Hashing 1.00 MB message, producing 32.00 B digest   428.3 µs      │ 1.738 ms      │ 597.2 µs      │ 634.8 µs      │ 100     │ 100
   │                                                      2.279 GiB/s   │ 575.1 MiB/s   │ 1.635 GiB/s   │ 1.538 GiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        167         │ 167           │ 167           │ 167           │         │
   │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        168         │ 168           │ 168           │ 168           │         │
   │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 4             │         │
   │                                                        9 KiB       │ 9 KiB         │ 9 KiB         │ 9 KiB         │         │
   ├─ Hashing 32.00 B message, producing 32.00 B digest   355.8 ns      │ 18.09 µs      │ 657.3 ns      │ 895.1 ns      │ 100     │ 100
   │                                                      171.5 MiB/s   │ 3.373 MiB/s   │ 92.84 MiB/s   │ 68.18 MiB/s   │         │
   ├─ Hashing 32.00 KB message, producing 32.00 B digest  82.38 µs      │ 252.4 µs      │ 127.6 µs      │ 127.5 µs      │ 100     │ 100
   │                                                      379.6 MiB/s   │ 123.9 MiB/s   │ 245 MiB/s     │ 245.3 MiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        47          │ 47            │ 47            │ 47            │         │
   │                                                        15.32 KiB   │ 15.32 KiB     │ 15.32 KiB     │ 15.32 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        48          │ 48            │ 48            │ 48            │         │
   │                                                        15.39 KiB   │ 15.39 KiB     │ 15.39 KiB     │ 15.39 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 4             │         │
   │                                                        512 B       │ 512 B         │ 512 B         │ 512 B         │         │
   ├─ Hashing 32.00 MB message, producing 32.00 B digest  4.661 ms      │ 7.196 ms      │ 5.695 ms      │ 5.645 ms      │ 100     │ 100
   │                                                      6.703 GiB/s   │ 4.342 GiB/s   │ 5.486 GiB/s   │ 5.535 GiB/s   │         │
   │                                                      max alloc:    │               │               │               │         │
   │                                                        167         │ 167           │ 167           │ 167           │         │
   │                                                        54.98 KiB   │ 54.98 KiB     │ 54.98 KiB     │ 54.98 KiB     │         │
   │                                                      alloc:        │               │               │               │         │
   │                                                        168         │ 168           │ 168           │ 168           │         │
   │                                                        55.23 KiB   │ 55.23 KiB     │ 55.23 KiB     │ 55.23 KiB     │         │
   │                                                      dealloc:      │               │               │               │         │
   │                                                        4           │ 4             │ 4             │ 5.01          │         │
   │                                                        257 KiB     │ 257 KiB       │ 257 KiB       │ 257.4 KiB     │         │
   ╰─ Hashing 64.00 B message, producing 32.00 B digest   370.8 ns      │ 3.293 µs      │ 384.3 ns      │ 451.8 ns      │ 100     │ 100
                                                          246.8 MiB/s   │ 27.79 MiB/s   │ 238.1 MiB/s   │ 202.6 MiB/s   │         │
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
// Following example demonstrates how to use KT128, and similarly you can use KT256.

use kangarootwelve::KT128;
use rand::{rng, RngCore};

fn main() {
  const MLEN: usize = 64;
  const CSTRLEN: usize = 1;
  const DLEN: usize = 32;

  let mut msg = vec![0u8; MLEN];
  let mut cstr = vec![0u8; CSTRLEN]; // you can keep it empty
  let mut dig = vec![0u8; DLEN];

  let mut rng = rng();
  rng.fill_bytes(&mut msg);
  cstr[0] = 0xff;

  let mut hasher = KT128::hash(&msg, &cstr);
  // ...
}
```

3) Sponge is ready to be squeezed i.e. now you can use returned XOF object for squeezing arbitrary number of bytes arbitrary number of times.

```rust
hasher.squeeze(&mut dig[..DLEN / 2]);
hasher.squeeze(&mut dig[DLEN / 2..]);
```

I maintain [examples](./examples/), demonstrating usage of KangarooTwelve eXtendable Output Function (XOF). Execute them by running `$ make example`.

```
Using KT128
Message              = b2551f09169df9e10314acf7e8bb81af46a68c4748c49473da704d9386f871085272d3313afe96d51889ad9c2a1628c4f68ef00bf7dec89abf70204c9b778c84
Customization String = ff
Digest               = 2e93ed342a89def8c75721295206d68d4518838fdb7dfb11985d581c914a2afb

Using KT256
Message              = 1c71343c1b76032836db92ff8a5121e66aa62ce0111b28504615411f897a6dcc9af53edede9ed46ee0e41d19338eb3a5dd79bf1fda123eba3507bc9e5f04d76b
Customization String = ff
Digest               = fdda23356a3111dd01867dcbe3a874303f6ece12f04dc506e6cf3bc88db7cf5d
```
