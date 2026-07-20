#pragma once

#include <cstddef>
#include <cstdint>

#include "keccak.cuh"
#include "utils.cuh"

namespace kangarootwelve::sponge {

__host__ __device__ inline void
absorb(uint64_t state[keccak::LANE_COUNT], size_t* offset, const uint8_t* msg, size_t len, size_t rate)
{
  size_t off = *offset;

  size_t i = 0;
  while (i < len) {
    size_t space = rate - off;
    size_t rem = len - i;
    size_t take = (rem < space) ? rem : space;

    size_t j = 0;
    while (j < take && (off & 7)) {
      state[off >> 3] ^= (uint64_t)msg[i + j] << (8 * (off & 7));

      off++;
      j++;
    }

    while (j + 8 <= take) {
      state[off >> 3] ^= utils::u64_le_load(msg + i + j);
      off += 8;
      j += 8;
    }

    while (j < take) {
      state[off >> 3] ^= (uint64_t)msg[i + j] << (8 * (off & 7));
      off++;
      j++;
    }

    i += take;
    if (off == rate) {
      keccak::permute(state);
      off = 0;
    }
  }

  *offset = off;
}

__host__ __device__ inline void
finalize(uint64_t state[keccak::LANE_COUNT], size_t* offset, size_t rate, uint8_t D)
{
  state[(*offset) >> 3] ^= (uint64_t)D << (8 * ((*offset) & 7));
  state[(rate - 1) >> 3] ^= (uint64_t)0x80 << (8 * ((rate - 1) & 7));

  keccak::permute(state);
  *offset = 0;
}

__host__ __device__ inline void
squeeze(uint64_t state[keccak::LANE_COUNT], size_t* offset, uint8_t* out, size_t outlen, size_t rate)
{
  size_t off = *offset;

  size_t i = 0;
  while (i < outlen) {
    size_t readable = rate - off;
    size_t rem = outlen - i;
    size_t take = (rem < readable) ? rem : readable;

    for (size_t k = 0; k < take; k++) {
      size_t p = off + k;
      out[i + k] = (uint8_t)(state[p >> 3] >> (8 * (p & 7)));
    }

    i += take;
    off += take;

    if (off == rate) {
      keccak::permute(state);
      off = 0;
    }
  }

  *offset = off;
}

} // namespace kangarootwelve::sponge
