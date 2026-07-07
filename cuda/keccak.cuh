#pragma once

#include <cstddef>
#include <cstdint>

#include "utils.cuh"

namespace kangarootwelve::keccak {

constexpr size_t ROUNDS = 12;
constexpr size_t LANE_COUNT = 25;

__host__ __device__ inline void
permute(uint64_t A[LANE_COUNT])
{
  constexpr uint64_t RC[ROUNDS] = { 0x000000008000808bULL, 0x800000000000008bULL, 0x8000000000008089ULL, 0x8000000000008003ULL,
                                    0x8000000000008002ULL, 0x8000000000000080ULL, 0x000000000000800aULL, 0x800000008000000aULL,
                                    0x8000000080008081ULL, 0x8000000000008080ULL, 0x0000000080000001ULL, 0x8000000080008008ULL };

#ifdef __CUDA_ARCH__
  // On GPU
  constexpr int ROT[LANE_COUNT] = { 0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14 };

  for (size_t r = 0; r < ROUNDS; r++) {
    uint64_t C[5], D[5], B[LANE_COUNT];

    // theta
#pragma unroll
    for (size_t x = 0; x < 5; x++) {
      C[x] = A[x] ^ A[x + 5] ^ A[x + 10] ^ A[x + 15] ^ A[x + 20];
    }
#pragma unroll
    for (size_t x = 0; x < 5; x++) {
      D[x] = C[(x + 4) % 5] ^ utils::rotl64(C[(x + 1) % 5], 1);
    }
#pragma unroll
    for (size_t i = 0; i < LANE_COUNT; i++) {
      A[i] ^= D[i % 5];
    }

    // rho + pi
#pragma unroll
    for (size_t x = 0; x < 5; x++)
#pragma unroll
      for (size_t y = 0; y < 5; y++) {
        const size_t src = x + 5 * y;
        const size_t dst = y + 5 * ((2 * x + 3 * y) % 5);

        B[dst] = utils::rotl64(A[src], ROT[src]);
      }

    // chi
#pragma unroll
    for (size_t y = 0; y < 5; y++) {
#pragma unroll
      for (size_t x = 0; x < 5; x++) {
        A[x + 5 * y] = B[x + 5 * y] ^ ((~B[(x + 1) % 5 + 5 * y]) & B[(x + 2) % 5 + 5 * y]);
      }
    }

    // iota
    A[0] ^= RC[r];
  }
#else
  // On CPU
  uint64_t a0 = A[0], a1 = A[1], a2 = A[2], a3 = A[3], a4 = A[4];
  uint64_t a5 = A[5], a6 = A[6], a7 = A[7], a8 = A[8], a9 = A[9];
  uint64_t a10 = A[10], a11 = A[11], a12 = A[12], a13 = A[13], a14 = A[14];
  uint64_t a15 = A[15], a16 = A[16], a17 = A[17], a18 = A[18], a19 = A[19];
  uint64_t a20 = A[20], a21 = A[21], a22 = A[22], a23 = A[23], a24 = A[24];

  for (size_t r = 0; r < ROUNDS; r++) {
    const uint64_t c0 = a0 ^ a5 ^ a10 ^ a15 ^ a20;
    const uint64_t c1 = a1 ^ a6 ^ a11 ^ a16 ^ a21;
    const uint64_t c2 = a2 ^ a7 ^ a12 ^ a17 ^ a22;
    const uint64_t c3 = a3 ^ a8 ^ a13 ^ a18 ^ a23;
    const uint64_t c4 = a4 ^ a9 ^ a14 ^ a19 ^ a24;

    const uint64_t d0 = c4 ^ utils::rotl64(c1, 1);
    const uint64_t d1 = c0 ^ utils::rotl64(c2, 1);
    const uint64_t d2 = c1 ^ utils::rotl64(c3, 1);
    const uint64_t d3 = c2 ^ utils::rotl64(c4, 1);
    const uint64_t d4 = c3 ^ utils::rotl64(c0, 1);

    a0 ^= d0;
    a5 ^= d0;
    a10 ^= d0;
    a15 ^= d0;
    a20 ^= d0;
    a1 ^= d1;
    a6 ^= d1;
    a11 ^= d1;
    a16 ^= d1;
    a21 ^= d1;
    a2 ^= d2;
    a7 ^= d2;
    a12 ^= d2;
    a17 ^= d2;
    a22 ^= d2;
    a3 ^= d3;
    a8 ^= d3;
    a13 ^= d3;
    a18 ^= d3;
    a23 ^= d3;
    a4 ^= d4;
    a9 ^= d4;
    a14 ^= d4;
    a19 ^= d4;
    a24 ^= d4;

    const uint64_t b0 = a0;
    const uint64_t b1 = utils::rotl64(a6, 44);
    const uint64_t b2 = utils::rotl64(a12, 43);
    const uint64_t b3 = utils::rotl64(a18, 21);
    const uint64_t b4 = utils::rotl64(a24, 14);
    const uint64_t b5 = utils::rotl64(a3, 28);
    const uint64_t b6 = utils::rotl64(a9, 20);
    const uint64_t b7 = utils::rotl64(a10, 3);
    const uint64_t b8 = utils::rotl64(a16, 45);
    const uint64_t b9 = utils::rotl64(a22, 61);
    const uint64_t b10 = utils::rotl64(a1, 1);
    const uint64_t b11 = utils::rotl64(a7, 6);
    const uint64_t b12 = utils::rotl64(a13, 25);
    const uint64_t b13 = utils::rotl64(a19, 8);
    const uint64_t b14 = utils::rotl64(a20, 18);
    const uint64_t b15 = utils::rotl64(a4, 27);
    const uint64_t b16 = utils::rotl64(a5, 36);
    const uint64_t b17 = utils::rotl64(a11, 10);
    const uint64_t b18 = utils::rotl64(a17, 15);
    const uint64_t b19 = utils::rotl64(a23, 56);
    const uint64_t b20 = utils::rotl64(a2, 62);
    const uint64_t b21 = utils::rotl64(a8, 55);
    const uint64_t b22 = utils::rotl64(a14, 39);
    const uint64_t b23 = utils::rotl64(a15, 41);
    const uint64_t b24 = utils::rotl64(a21, 2);

    a0 = b0 ^ ((~b1) & b2);
    a1 = b1 ^ ((~b2) & b3);
    a2 = b2 ^ ((~b3) & b4);
    a3 = b3 ^ ((~b4) & b0);
    a4 = b4 ^ ((~b0) & b1);
    a5 = b5 ^ ((~b6) & b7);
    a6 = b6 ^ ((~b7) & b8);
    a7 = b7 ^ ((~b8) & b9);
    a8 = b8 ^ ((~b9) & b5);
    a9 = b9 ^ ((~b5) & b6);
    a10 = b10 ^ ((~b11) & b12);
    a11 = b11 ^ ((~b12) & b13);
    a12 = b12 ^ ((~b13) & b14);
    a13 = b13 ^ ((~b14) & b10);
    a14 = b14 ^ ((~b10) & b11);
    a15 = b15 ^ ((~b16) & b17);
    a16 = b16 ^ ((~b17) & b18);
    a17 = b17 ^ ((~b18) & b19);
    a18 = b18 ^ ((~b19) & b15);
    a19 = b19 ^ ((~b15) & b16);
    a20 = b20 ^ ((~b21) & b22);
    a21 = b21 ^ ((~b22) & b23);
    a22 = b22 ^ ((~b23) & b24);
    a23 = b23 ^ ((~b24) & b20);
    a24 = b24 ^ ((~b20) & b21);

    a0 ^= RC[r];
  }

  A[0] = a0, A[1] = a1, A[2] = a2, A[3] = a3, A[4] = a4;
  A[5] = a5, A[6] = a6, A[7] = a7, A[8] = a8, A[9] = a9;
  A[10] = a10, A[11] = a11, A[12] = a12, A[13] = a13, A[14] = a14;
  A[15] = a15, A[16] = a16, A[17] = a17, A[18] = a18, A[19] = a19;
  A[20] = a20, A[21] = a21, A[22] = a22, A[23] = a23, A[24] = a24;
#endif
}

} // namespace kangarootwelve::keccak
