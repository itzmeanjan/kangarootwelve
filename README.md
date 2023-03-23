# kangarootwelve
BlaKE12: Blazing-fast KEccak on 12 rounds

## Overview

KangarooTwelve is a fast and secure arbitrary output-length hash function which performs much better than hash and extendable output functions specified on FIPS 202 ( more @ https://dx.doi.org/10.6028/NIST.FIPS.202 ). KangarooTwelve ( aka K12 or BlaKE12 - see more @ https://blake12.org ) is built on top of round-reduced keccak-p[1600, 12] permutation. To be more specific KangarooTwelve can be implemented on top of TurboSHAKE - which is a family of extendable output functions, recently specified on https://ia.cr/2023/342. The sponge mode of KangarooTwelve uses 256 -bit wide capacity i.e. it can use TurboSHAKE128 as underlying construction. K12 possesses a built-in parallel hashing mode for long ( >=8KB ) messages which can efficiently be exploited by multiple-cores or SIMD instructions. Another important gain of K12 is that its parallel design doesn't impact performance when hashing short ( <8KB ) messages. KangarooTwelve is specified on https://keccak.team/files/KangarooTwelve.pdf.

Here I'm developing/ maintaining a Rust library which implements KangarooTwelve specification s.t. it implements non-incremental absorption API with arbitrary times squeeze support. In coming weeks, I plan to support incremental hashing API i.e. one can construct a K12 hasher object for absorbing message bytes arbitrary many times, then finalize using customization string, before squeezing bytes out of sponge state. See [below](#usage) for example, showing usage of K12 XOF API.

## Prerequisites

Rust stable toolchain; see https://rustup.rs for installation guide.

```bash
# When developing this library, I was using
$ rustc --version
rustc 1.68.0 (2c8cc3432 2023-03-06)
```

## Testing

For ensuring functional correctness of KangarooTwelve XOF's implementation, I use test vectors from section 4 ( on page 9 ) and Appendix A ( on page 17 ) of https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve. Issue following command to run test cases.

```bash
cargo test --lib # by default it's single-threaded
cargo test --lib --features multi_threaded
```

## Benchmarking

Issue following command for benchmarking KangarooTwelve extendable output function's (XOF) implementation, for varying input sizes and fixed squeezed output size ( = 32 -bytes ).

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo bench # single-threaded absorption
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo bench --features multi_threaded # multi-threaded absorption
```

### On **Intel(R) Core(TM) i5-8279U CPU @ 2.40GHz** ( single-threaded )

```bash
K12/1024/32 (cached)    time:   [1.5607 µs 1.5674 µs 1.5746 µs]
                        thrpt:  [620.20 MiB/s 623.05 MiB/s 625.73 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
K12/1024/32 (random)    time:   [1.6377 µs 1.6485 µs 1.6613 µs]
                        thrpt:  [587.84 MiB/s 592.39 MiB/s 596.29 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

K12/2048/32 (cached)    time:   [2.5271 µs 2.5323 µs 2.5378 µs]
                        thrpt:  [769.61 MiB/s 771.29 MiB/s 772.86 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
K12/2048/32 (random)    time:   [2.7143 µs 2.7763 µs 2.8618 µs]
                        thrpt:  [682.49 MiB/s 703.51 MiB/s 719.57 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

K12/4096/32 (cached)    time:   [4.5423 µs 4.5512 µs 4.5609 µs]
                        thrpt:  [856.47 MiB/s 858.30 MiB/s 859.98 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
K12/4096/32 (random)    time:   [4.8527 µs 4.9012 µs 4.9540 µs]
                        thrpt:  [788.50 MiB/s 797.00 MiB/s 804.97 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild

K12/8192/32 (cached)    time:   [9.1117 µs 9.1528 µs 9.2036 µs]
                        thrpt:  [848.86 MiB/s 853.56 MiB/s 857.41 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
K12/8192/32 (random)    time:   [9.6723 µs 9.7596 µs 9.8542 µs]
                        thrpt:  [792.81 MiB/s 800.49 MiB/s 807.72 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  11 (11.00%) high mild
  1 (1.00%) high severe

K12/16384/32 (cached)   time:   [18.295 µs 18.404 µs 18.524 µs]
                        thrpt:  [843.51 MiB/s 849.01 MiB/s 854.07 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
K12/16384/32 (random)   time:   [18.790 µs 18.964 µs 19.173 µs]
                        thrpt:  [814.96 MiB/s 823.94 MiB/s 831.57 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

K12/32768/32 (cached)   time:   [35.931 µs 36.051 µs 36.189 µs]
                        thrpt:  [863.52 MiB/s 866.83 MiB/s 869.73 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
K12/32768/32 (random)   time:   [37.553 µs 39.232 µs 42.030 µs]
                        thrpt:  [743.52 MiB/s 796.54 MiB/s 832.15 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe

Benchmarking K12/1048576/32 (cached): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.9s, enable flat sampling, or reduce sample count to 60.
K12/1048576/32 (cached) time:   [1.1502 ms 1.1557 ms 1.1621 ms]
                        thrpt:  [860.54 MiB/s 865.31 MiB/s 869.43 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking K12/1048576/32 (random): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.7s, enable flat sampling, or reduce sample count to 50.
K12/1048576/32 (random) time:   [1.2591 ms 1.2692 ms 1.2807 ms]
                        thrpt:  [780.81 MiB/s 787.88 MiB/s 794.22 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

K12/2097152/32 (cached) time:   [2.3188 ms 2.3336 ms 2.3496 ms]
                        thrpt:  [851.21 MiB/s 857.05 MiB/s 862.53 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
K12/2097152/32 (random) time:   [2.5263 ms 2.5515 ms 2.5794 ms]
                        thrpt:  [775.36 MiB/s 783.87 MiB/s 791.68 MiB/s]
Found 17 outliers among 100 measurements (17.00%)
  2 (2.00%) high mild
  15 (15.00%) high severe

K12/4194304/32 (cached) time:   [4.6883 ms 4.7201 ms 4.7552 ms]
                        thrpt:  [841.19 MiB/s 847.43 MiB/s 853.20 MiB/s]
Found 17 outliers among 100 measurements (17.00%)
  13 (13.00%) high mild
  4 (4.00%) high severe
K12/4194304/32 (random) time:   [4.9172 ms 4.9354 ms 4.9555 ms]
                        thrpt:  [807.18 MiB/s 810.46 MiB/s 813.46 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

K12/8388608/32 (cached) time:   [9.6927 ms 9.7635 ms 9.8413 ms]
                        thrpt:  [812.90 MiB/s 819.38 MiB/s 825.36 MiB/s]
Found 15 outliers among 100 measurements (15.00%)
  6 (6.00%) high mild
  9 (9.00%) high severe
K12/8388608/32 (random) time:   [10.085 ms 10.110 ms 10.136 ms]
                        thrpt:  [789.28 MiB/s 791.32 MiB/s 793.23 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild

K12/16777216/32 (cached)
                        time:   [19.412 ms 19.465 ms 19.524 ms]
                        thrpt:  [819.51 MiB/s 821.99 MiB/s 824.23 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
K12/16777216/32 (random)
                        time:   [20.295 ms 20.372 ms 20.465 ms]
                        thrpt:  [781.83 MiB/s 785.39 MiB/s 788.37 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

K12/33554432/32 (cached)
                        time:   [38.981 ms 39.092 ms 39.212 ms]
                        thrpt:  [816.08 MiB/s 818.57 MiB/s 820.92 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
K12/33554432/32 (random)
                        time:   [40.963 ms 41.106 ms 41.270 ms]
                        thrpt:  [775.38 MiB/s 778.48 MiB/s 781.19 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
```

### On **Intel(R) Core(TM) i5-8279U CPU @ 2.40GHz** ( multi-threaded )

```bash
K12/1024/32 (cached)    time:   [1.5234 µs 1.5303 µs 1.5377 µs]
                        thrpt:  [635.06 MiB/s 638.17 MiB/s 641.04 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
K12/1024/32 (random)    time:   [1.7047 µs 1.7214 µs 1.7407 µs]
                        thrpt:  [561.03 MiB/s 567.31 MiB/s 572.85 MiB/s]
Found 20 outliers among 100 measurements (20.00%)
  5 (5.00%) low mild
  6 (6.00%) high mild
  9 (9.00%) high severe

K12/2048/32 (cached)    time:   [2.4866 µs 2.4927 µs 2.4994 µs]
                        thrpt:  [781.45 MiB/s 783.53 MiB/s 785.47 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
K12/2048/32 (random)    time:   [2.7111 µs 2.7326 µs 2.7558 µs]
                        thrpt:  [708.72 MiB/s 714.76 MiB/s 720.42 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  11 (11.00%) high mild
  1 (1.00%) high severe

K12/4096/32 (cached)    time:   [4.4709 µs 4.4844 µs 4.4995 µs]
                        thrpt:  [868.14 MiB/s 871.08 MiB/s 873.70 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
K12/4096/32 (random)    time:   [4.9004 µs 4.9472 µs 4.9969 µs]
                        thrpt:  [781.73 MiB/s 789.60 MiB/s 797.13 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

K12/8192/32 (cached)    time:   [51.266 µs 53.230 µs 56.058 µs]
                        thrpt:  [139.36 MiB/s 146.77 MiB/s 152.39 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
K12/8192/32 (random)    time:   [52.633 µs 53.242 µs 53.895 µs]
                        thrpt:  [144.96 MiB/s 146.74 MiB/s 148.43 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

K12/16384/32 (cached)   time:   [65.168 µs 65.468 µs 65.911 µs]
                        thrpt:  [237.06 MiB/s 238.67 MiB/s 239.76 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
K12/16384/32 (random)   time:   [66.082 µs 66.230 µs 66.393 µs]
                        thrpt:  [235.34 MiB/s 235.92 MiB/s 236.45 MiB/s]
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) low mild
  8 (8.00%) high mild
  8 (8.00%) high severe

K12/32768/32 (cached)   time:   [103.50 µs 103.99 µs 104.56 µs]
                        thrpt:  [298.87 MiB/s 300.52 MiB/s 301.94 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
K12/32768/32 (random)   time:   [104.99 µs 106.87 µs 109.35 µs]
                        thrpt:  [285.78 MiB/s 292.41 MiB/s 297.65 MiB/s]
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) high mild
  8 (8.00%) high severe

K12/1048576/32 (cached) time:   [595.11 µs 600.65 µs 606.11 µs]
                        thrpt:  [1.6112 GiB/s 1.6258 GiB/s 1.6410 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking K12/1048576/32 (random): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.5s, enable flat sampling, or reduce sample count to 60.
K12/1048576/32 (random) time:   [644.87 µs 652.94 µs 660.56 µs]
                        thrpt:  [1.4784 GiB/s 1.4956 GiB/s 1.5144 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild

K12/2097152/32 (cached) time:   [963.57 µs 970.18 µs 977.03 µs]
                        thrpt:  [1.9990 GiB/s 2.0132 GiB/s 2.0270 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking K12/2097152/32 (random): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.7s, enable flat sampling, or reduce sample count to 50.
K12/2097152/32 (random) time:   [1.0573 ms 1.0669 ms 1.0769 ms]
                        thrpt:  [1.8136 GiB/s 1.8306 GiB/s 1.8473 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking K12/4194304/32 (cached): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.6s, enable flat sampling, or reduce sample count to 50.
K12/4194304/32 (cached) time:   [1.6748 ms 1.6845 ms 1.6939 ms]
                        thrpt:  [2.3060 GiB/s 2.3189 GiB/s 2.3324 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
K12/4194304/32 (random) time:   [1.8045 ms 1.8168 ms 1.8295 ms]
                        thrpt:  [2.1351 GiB/s 2.1501 GiB/s 2.1647 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

K12/8388608/32 (cached) time:   [3.1208 ms 3.1417 ms 3.1636 ms]
                        thrpt:  [2.4695 GiB/s 2.4867 GiB/s 2.5033 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
K12/8388608/32 (random) time:   [3.2660 ms 3.2813 ms 3.2974 ms]
                        thrpt:  [2.3693 GiB/s 2.3810 GiB/s 2.3920 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe

K12/16777216/32 (cached)
                        time:   [5.9027 ms 5.9612 ms 6.0277 ms]
                        thrpt:  [2.5922 GiB/s 2.6211 GiB/s 2.6471 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
K12/16777216/32 (random)
                        time:   [6.2938 ms 6.3308 ms 6.3723 ms]
                        thrpt:  [2.4520 GiB/s 2.4681 GiB/s 2.4826 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

K12/33554432/32 (cached)
                        time:   [11.659 ms 11.741 ms 11.829 ms]
                        thrpt:  [2.6417 GiB/s 2.6617 GiB/s 2.6802 GiB/s]
Found 21 outliers among 100 measurements (21.00%)
  7 (7.00%) high mild
  14 (14.00%) high severe
K12/33554432/32 (random)
                        time:   [12.109 ms 12.183 ms 12.266 ms]
                        thrpt:  [2.5478 GiB/s 2.5651 GiB/s 2.5808 GiB/s]
Found 14 outliers among 100 measurements (14.00%)
  9 (9.00%) high mild
  5 (5.00%) high severe
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
