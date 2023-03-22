# kangarootwelve
BlaKE12: Blazing-fast KEccak on 12 rounds

## Testing

For ensuring functional correctness of KangarooTwelve XOF's single threaded implementation, I use test vectors from section 4 ( on page 9 ) and Appendix A ( on page 17 ) of https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve. Issue following command to run test cases.

```bash
cargo test --lib
```

## Benchmarking

Issue following command for benchmarking KangarooTwelve extendable output function's (XOF) single threaded implementation, for varying input sizes and fixed squeezed output size ( = 32 -bytes ).

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo bench
```

```bash
K12/32/32 (cached)      time:   [430.62 ns 432.65 ns 434.95 ns]
                        thrpt:  [70.163 MiB/s 70.537 MiB/s 70.868 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
K12/32/32 (random)      time:   [459.54 ns 461.78 ns 464.22 ns]
                        thrpt:  [65.739 MiB/s 66.087 MiB/s 66.409 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

K12/64/32 (cached)      time:   [429.17 ns 431.44 ns 433.87 ns]
                        thrpt:  [140.68 MiB/s 141.47 MiB/s 142.22 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
K12/64/32 (random)      time:   [470.64 ns 474.01 ns 477.53 ns]
                        thrpt:  [127.81 MiB/s 128.76 MiB/s 129.69 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

K12/128/32 (cached)     time:   [429.85 ns 431.44 ns 433.25 ns]
                        thrpt:  [281.76 MiB/s 282.93 MiB/s 283.99 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
K12/128/32 (random)     time:   [476.86 ns 480.85 ns 485.13 ns]
                        thrpt:  [251.62 MiB/s 253.86 MiB/s 255.99 MiB/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe

K12/256/32 (cached)     time:   [605.48 ns 607.87 ns 610.47 ns]
                        thrpt:  [399.92 MiB/s 401.63 MiB/s 403.22 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
K12/256/32 (random)     time:   [682.99 ns 691.59 ns 703.27 ns]
                        thrpt:  [347.15 MiB/s 353.02 MiB/s 357.46 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe

K12/512/32 (cached)     time:   [942.80 ns 945.94 ns 949.39 ns]
                        thrpt:  [514.31 MiB/s 516.19 MiB/s 517.91 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
K12/512/32 (random)     time:   [1.1085 µs 1.1233 µs 1.1388 µs]
                        thrpt:  [428.76 MiB/s 434.67 MiB/s 440.50 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

K12/1024/32 (cached)    time:   [1.4491 µs 1.4561 µs 1.4643 µs]
                        thrpt:  [666.91 MiB/s 670.65 MiB/s 673.91 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
K12/1024/32 (random)    time:   [1.5191 µs 1.5302 µs 1.5421 µs]
                        thrpt:  [633.28 MiB/s 638.21 MiB/s 642.85 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

K12/2048/32 (cached)    time:   [2.4527 µs 2.4606 µs 2.4694 µs]
                        thrpt:  [790.94 MiB/s 793.75 MiB/s 796.33 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
K12/2048/32 (random)    time:   [2.6173 µs 2.6527 µs 2.6910 µs]
                        thrpt:  [725.80 MiB/s 736.27 MiB/s 746.24 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

K12/4096/32 (cached)    time:   [4.4843 µs 4.5086 µs 4.5356 µs]
                        thrpt:  [861.23 MiB/s 866.39 MiB/s 871.10 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
K12/4096/32 (random)    time:   [4.7358 µs 4.7800 µs 4.8269 µs]
                        thrpt:  [809.27 MiB/s 817.20 MiB/s 824.84 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

K12/8192/32 (cached)    time:   [9.6841 µs 9.7966 µs 9.9702 µs]
                        thrpt:  [783.59 MiB/s 797.47 MiB/s 806.74 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
K12/8192/32 (random)    time:   [9.9037 µs 10.000 µs 10.103 µs]
                        thrpt:  [773.25 MiB/s 781.22 MiB/s 788.85 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

Benchmarking K12/1048576/32 (cached): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.8s, enable flat sampling, or reduce sample count to 60.
K12/1048576/32 (cached) time:   [1.1412 ms 1.1499 ms 1.1599 ms]
                        thrpt:  [862.13 MiB/s 869.63 MiB/s 876.27 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking K12/1048576/32 (random): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.7s, enable flat sampling, or reduce sample count to 50.
K12/1048576/32 (random) time:   [1.2520 ms 1.2617 ms 1.2718 ms]
                        thrpt:  [786.26 MiB/s 792.60 MiB/s 798.69 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

K12/2097152/32 (cached) time:   [2.2837 ms 2.3074 ms 2.3369 ms]
                        thrpt:  [855.82 MiB/s 866.77 MiB/s 875.78 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
K12/2097152/32 (random) time:   [2.4437 ms 2.4523 ms 2.4617 ms]
                        thrpt:  [812.45 MiB/s 815.56 MiB/s 818.43 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

K12/4194304/32 (cached) time:   [4.6105 ms 4.6529 ms 4.7004 ms]
                        thrpt:  [850.99 MiB/s 859.67 MiB/s 867.58 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
K12/4194304/32 (random) time:   [6.9595 ms 7.7759 ms 8.6692 ms]
                        thrpt:  [461.40 MiB/s 514.41 MiB/s 574.76 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

K12/8388608/32 (cached) time:   [9.4836 ms 9.5198 ms 9.5591 ms]
                        thrpt:  [836.90 MiB/s 840.36 MiB/s 843.56 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
K12/8388608/32 (random) time:   [9.8046 ms 9.8351 ms 9.8681 ms]
                        thrpt:  [810.69 MiB/s 813.41 MiB/s 815.95 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
```
