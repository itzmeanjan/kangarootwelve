# kangarootwelve

High-throughput parallel hashing with 12-rounds Keccak

## Overview

KangarooTwelve is a family of extendable output functions (XOFs) based on 12-rounds Keccak-p[1600] permutation.
It implements a tree hashing mode, following SAKURA coding <https://eprint.iacr.org/2013/231>.
The message absorption phase is parallelizable - designed to exploit multi-threading or SIMD parallelism.
Naturally it performs much better than hash functions specified in FIPS 202, i.e., SHA3 standard.
KangarooTwelve is specified in RFC 9861 <https://www.rfc-editor.org/info/rfc9861/>.
It has two instances.

- KT128, uses TurboSHAKE128 for hashing chunks. It offers up to 128 bits of collision resistance security.
- KT256, uses TurboSHAKE256 for hashing chunks. It offers up to 256 bits of collision resistance security.

Here I'm developing and maintaining a Rust library crate which implements KangarooTwelve.
For now it does not expose a streamed hashing interface.
Meaning the full message needs to be in-memory before it can be hashed.
The returned object from hashing the message lets you squeeze arbitrary long output.
This library lets you perform hashing either on CPU or NVIDIA GPUs.
Both multi-threaded hashing on a CPU and hashing on a NVIDIA GPU is feature-gated.
Offloading hashing to GPUs, manufactured by other vendors, is not yet supported.

KT128 XOF achieves a hashing throughput of ~215GiB/s on a data center-grade NVIDIA GPU.
Multi-threaded hashing throughput on CPU systems largely depends on the number of logical cores.
Achievable memory bandwidth affects hashing throughput on both CPU and GPU platforms.

Platform | Throughput | Saturates for messages
--- | --- | ---
NVIDIA RTX PRO 6000 Blackwell Server Edition | 215GiB/s | >=16GiB
Intel Xeon Platinum 8559C | 4.8GiB/s | >=1GiB
Intel Core i7-1260P | 7.4GiB/s | >=1GiB

See [below](#usage) examples, showing how to use the library.

## Prerequisites

Rust stable toolchain. See <https://rustup.rs> for installation guide. MSRV for this crate is 1.86.0.
If you want to run the tests in WebAssembly environment, you need to have `wasmtime-cli@35.0.0` installed.
You will need to add `wasm32-wasip1` and `wasm32-wasip2` targets so that Rust toolchain can produce WebAssembly executable code.

```bash
rustup target add wasm32-wasip1
rustup target add wasm32-wasip2
cargo install wasmtime-cli@35.0.0 --locked
```

## Testing

For ensuring functional correctness and conformance to RFC 9861, we use test vectors from section 5 of RFC 9861 <https://www.rfc-editor.org/rfc/rfc9861.html>.
Issue following command to run all tests.

```bash
make test      # Single-threaded backend
make test-mt   # Multi-threaded backend
make test-cuda # NVIDIA GPUs
make test-wasm # Single-threaded backend inside WebAssembly environment
```

## Benchmarking

Benchmark performance of KangarooTwelve family of XOFs, for variable input length and fixed squeezed output length, on both CPU and GPU.

```bash
make bench      # For CPUs, single-threaded
make bench-mt   # For CPUs, multi-threaded
make bench-cuda # For NVIDIA GPUs
```

<details>
<summary>Click to view detailed benchmark results</summary>

> [!WARNING]
> When benchmarking make sure you've disabled CPU frequency scaling, otherwise numbers you see can be misleading. I find the guide @ <https://github.com/google/benchmark/blob/b40db869/docs/reducing_variance.md> helpful.

### On NVIDIA RTX PRO 6000 Blackwell Server Edition [AWS EC2 g7e.2xlarge]

> [!INFO]
> It is a data center-grade GPU with compute capability (CC) of 12.0.
> More on CC @ <https://developer.nvidia.com/cuda/gpus>.
> Use `$ nvidia-smi --query-gpu=compute_cap --format=csv,noheader` to get compute capability of any NVIDIA GPU.

```bash
# Output of running `make bench-cuda`

kt128/hashing 1.00 KB message, producing 32.00 B digest                                                                            
                        time:   [41.847 µs 41.923 µs 41.991 µs]
                        thrpt:  [23.983 MiB/s 24.022 MiB/s 24.066 MiB/s]
kt128/hashing 1.00 MB message, producing 32.00 B digest                                                                           
                        time:   [734.14 µs 734.22 µs 734.28 µs]
                        thrpt:  [1.3300 GiB/s 1.3301 GiB/s 1.3303 GiB/s]
kt128/hashing 1.00 GB message, producing 32.00 B digest                                                                           
                        time:   [6.7707 ms 6.7727 ms 6.7741 ms]
                        thrpt:  [147.62 GiB/s 147.65 GiB/s 147.70 GiB/s]
kt128/hashing 4.00 GB message, producing 32.00 B digest                                                                           
                        time:   [20.211 ms 20.228 ms 20.244 ms]
                        thrpt:  [197.59 GiB/s 197.75 GiB/s 197.91 GiB/s]
kt128/hashing 16.00 GB message, producing 32.00 B digest                                                                          
                        time:   [74.477 ms 74.552 ms 74.649 ms]
                        thrpt:  [214.34 GiB/s 214.61 GiB/s 214.83 GiB/s]
kt128/hashing 32.00 GB message, producing 32.00 B digest                                                          
                        time:   [145.42 ms 145.64 ms 145.85 ms]
                        thrpt:  [219.41 GiB/s 219.72 GiB/s 220.06 GiB/s]

# ---

kt256/hashing 1.00 KB message, producing 32.00 B digest                                                                            
                        time:   [47.315 µs 47.320 µs 47.323 µs]
                        thrpt:  [21.281 MiB/s 21.282 MiB/s 21.285 MiB/s]
kt256/hashing 1.00 MB message, producing 32.00 B digest                                                                           
                        time:   [772.98 µs 773.28 µs 773.89 µs]
                        thrpt:  [1.2619 GiB/s 1.2629 GiB/s 1.2634 GiB/s]
kt256/hashing 1.00 GB message, producing 32.00 B digest                                                                           
                        time:   [13.185 ms 13.191 ms 13.198 ms]
                        thrpt:  [75.769 GiB/s 75.812 GiB/s 75.845 GiB/s]
kt256/hashing 4.00 GB message, producing 32.00 B digest                                                                           
                        time:   [45.747 ms 45.759 ms 45.783 ms]
                        thrpt:  [87.368 GiB/s 87.414 GiB/s 87.438 GiB/s]
kt256/hashing 16.00 GB message, producing 32.00 B digest                                                                          
                        time:   [176.36 ms 176.51 ms 176.64 ms]
                        thrpt:  [90.579 GiB/s 90.648 GiB/s 90.723 GiB/s]
kt256/hashing 32.00 GB message, producing 32.00 B digest                                                                          
                        time:   [349.48 ms 349.76 ms 350.01 ms]
                        thrpt:  [91.426 GiB/s 91.493 GiB/s 91.563 GiB/s]
```

### On Intel(R) Xeon(R) Platinum 8559C [AWS EC2 g7e.2xlarge]

```bash
# Output of running `make bench` (only `multi_threaded` feature)

kt128/hashing 1.00 KB message, producing 32.00 B digest                                                                           
                        time:   [888.91 ns 889.50 ns 889.75 ns]
                        thrpt:  [1.1053 GiB/s 1.1057 GiB/s 1.1064 GiB/s]
kt128/hashing 1.00 MB message, producing 32.00 B digest                                                                           
                        time:   [408.57 µs 409.62 µs 411.21 µs]
                        thrpt:  [2.3749 GiB/s 2.3842 GiB/s 2.3903 GiB/s]
kt128/hashing 1.00 GB message, producing 32.00 B digest                                                                           
                        time:   [206.82 ms 207.23 ms 207.86 ms]
                        thrpt:  [4.8108 GiB/s 4.8256 GiB/s 4.8351 GiB/s]
kt128/hashing 4.00 GB message, producing 32.00 B digest                                                                          
                        time:   [824.02 ms 824.74 ms 825.42 ms]
                        thrpt:  [4.8460 GiB/s 4.8500 GiB/s 4.8542 GiB/s]

# ---

kt256/hashing 1.00 KB message, producing 32.00 B digest                                                                           
                        time:   [995.13 ns 995.32 ns 995.43 ns]
                        thrpt:  [1011.7 MiB/s 1011.8 MiB/s 1012.0 MiB/s]
kt256/hashing 1.00 MB message, producing 32.00 B digest                                                                           
                        time:   [464.92 µs 465.75 µs 466.58 µs]
                        thrpt:  [2.0931 GiB/s 2.0968 GiB/s 2.1006 GiB/s]
kt256/hashing 1.00 GB message, producing 32.00 B digest                                                                           
                        time:   [256.74 ms 257.06 ms 257.29 ms]
                        thrpt:  [3.8867 GiB/s 3.8901 GiB/s 3.8950 GiB/s]
kt256/hashing 4.00 GB message, producing 32.00 B digest                                                                          
                        time:   [1.0263 s 1.0275 s 1.0287 s]
                        thrpt:  [3.8884 GiB/s 3.8931 GiB/s 3.8973 GiB/s]
```

### On 12th Gen Intel(R) Core(TM) i7-1260P

> [!INFO]
> It is a mobile CPU with more details on Intel product specification page <https://www.intel.com/content/www/us/en/products/sku/226254/intel-core-i71260p-processor-18m-cache-up-to-4-70-ghz/specifications.html>.

```bash
# Output of running `make bench` (only `multi_threaded` feature)

kt128/hashing 1.00 KB message, producing 32.00 B digest                                                                           
                        time:   [792.06 ns 793.23 ns 795.50 ns]
                        thrpt:  [1.2363 GiB/s 1.2398 GiB/s 1.2417 GiB/s]
kt128/hashing 1.00 MB message, producing 32.00 B digest                                                                           
                        time:   [468.74 µs 470.14 µs 470.99 µs]
                        thrpt:  [2.0735 GiB/s 2.0773 GiB/s 2.0834 GiB/s]
kt128/hashing 1.00 GB message, producing 32.00 B digest                                                                           
                        time:   [135.31 ms 135.49 ms 135.65 ms]
                        thrpt:  [7.3717 GiB/s 7.3806 GiB/s 7.3904 GiB/s]
kt128/hashing 4.00 GB message, producing 32.00 B digest                                                                          
                        time:   [535.73 ms 538.41 ms 543.03 ms]
                        thrpt:  [7.3661 GiB/s 7.4293 GiB/s 7.4665 GiB/s]

# ---

kt256/hashing 1.00 KB message, producing 32.00 B digest                                                                           
                        time:   [934.08 ns 950.78 ns 964.35 ns]
                        thrpt:  [1.0198 GiB/s 1.0344 GiB/s 1.0529 GiB/s]
kt256/hashing 1.00 MB message, producing 32.00 B digest                                                                           
                        time:   [510.60 µs 514.63 µs 518.37 µs]
                        thrpt:  [1.8840 GiB/s 1.8976 GiB/s 1.9127 GiB/s]
kt256/hashing 1.00 GB message, producing 32.00 B digest                                                                           
                        time:   [162.27 ms 162.85 ms 163.33 ms]
                        thrpt:  [6.1226 GiB/s 6.1407 GiB/s 6.1624 GiB/s]
kt256/hashing 4.00 GB message, producing 32.00 B digest                                                                          
                        time:   [649.77 ms 653.72 ms 658.44 ms]
                        thrpt:  [6.0749 GiB/s 6.1188 GiB/s 6.1560 GiB/s]
```

</details>

## Usage

Getting started with using KangarooTwelve family of XOFs is easy

1) Add `kangarootwelve` as project dependency in your `Cargo.toml` file

```toml
[dependencies]
# either
kangarootwelve = { git = "https://github.com/itzmeanjan/kangarootwelve" }
# or
kangarootwelve = "0.1.3"
# or if interested in using multiple CPU threads for faster absorption
kangarootwelve = { version = "0.1.3", features = "multi_threaded" }
# or if interested in offloading hashing to NVIDIA GPUs
kangarootwelve = { version = "0.1.3", features = "cuda" }
```

2) For now both KT128 and KT256 offer only single-shot hashing API, i.e., full message should be ready in memory to be absorbed. It returns an XOF object, holding sponge in its finalized state.

```rust
// Following example demonstrates how to use KT128, and similarly you can use KT256.

use kangarootwelve::KT128;
use rand::{rng, RngCore};

fn main() {
  const MLEN: usize = 64;
  const CSTRLEN: usize = 1;
  const DLEN: usize = 32;

  let mut msg = vec![0u8; MLEN];
  let mut cstr = vec![0u8; CSTRLEN]; // This can be empty. One would generally use it for domain separation purposes.
  let mut dig = vec![0u8; DLEN];

  let mut rng = rng();
  rng.fill_bytes(&mut msg);
  cstr[0] = 0xff;

  let mut xof = KT128::hash(&msg, &cstr);
  // ...
}
```

3) Returned XOF object allows squeezing arbitrary long output stream.

```rust
xof.squeeze(&mut dig[..DLEN / 2]);
xof.squeeze(&mut dig[DLEN / 2..]);
```

---

I maintain couple of examples inside [examples/](./examples) directory. You can run them targeting different backends.

```bash
make example      # Run on single-threaded CPU environment
make example-mt   # Run on multi-threaded CPU environment
make example-cuda # Run on NVIDIA GPU
make example-wasm # Single-threaded backend inside WebAssembly environment
```

```bash
Using KT128
Message              = b2551f09169df9e10314acf7e8bb81af46a68c4748c49473da704d9386f871085272d3313afe96d51889ad9c2a1628c4f68ef00bf7dec89abf70204c9b778c84
Customization String = ff
Digest               = 2e93ed342a89def8c75721295206d68d4518838fdb7dfb11985d581c914a2afb

Using KT256
Message              = 1c71343c1b76032836db92ff8a5121e66aa62ce0111b28504615411f897a6dcc9af53edede9ed46ee0e41d19338eb3a5dd79bf1fda123eba3507bc9e5f04d76b
Customization String = ff
Digest               = fdda23356a3111dd01867dcbe3a874303f6ece12f04dc506e6cf3bc88db7cf5d
```

---

The CUDA backend is implemented in CUDA C++.
The Rust API exposed by this library crate is a wrapper around the underlying C++ implementation.
One might want to use the CUDA C++ API of KangarooTwelve family of XOFs.
See the example inside [cuda/examples/](./cuda/examples/) directory.
Run `make run` inside `cuda/` directory to execute the example.

```bash
message : "The quick brown fox jumps over the lazy dog"
KT128   : b4f249b4f77c58df170aa4d1723db1127d82f1d98d25ddda561ada459cd11a48
gpu absorb time : 454.996 us
```
