#pragma once

#include <cstddef>
#include <cstdint>
#include <cuda_runtime.h>

namespace kangarootwelve {

enum kt_status
{
  KT_OK = 0,
  KT_ERR_NO_DEVICE = 1,
  KT_ERR_ALLOC = 2,
  KT_ERR_MEMCPY = 3,
  KT_ERR_KERNEL = 4,
  KT_ERR_SYNC = 5,
  KT_ERR_GRID = 6,
};

} // namespace kangarootwelve

#define KT_TRY(expr, code)                                                                                                                                     \
  do {                                                                                                                                                         \
    cudaError_t _err = (expr);                                                                                                                                 \
    if (_err != cudaSuccess) {                                                                                                                                 \
      return (code);                                                                                                                                          \
    }                                                                                                                                                          \
  } while (0)

namespace kangarootwelve::utils {

// Rotate `x` left by `n` bits (0 <= n < 64).
__host__ __device__ __forceinline__ uint64_t
rotl64(uint64_t x, int n)
{
  if (n == 0) {
    return x;
  }

  return (x << n) | (x >> (64 - n));
}

__host__ __device__ __forceinline__ uint64_t
u64_le_load(const uint8_t* p)
{
  return (uint64_t)p[0] | ((uint64_t)p[1] << 8) | ((uint64_t)p[2] << 16) | ((uint64_t)p[3] << 24) | ((uint64_t)p[4] << 32) | ((uint64_t)p[5] << 40) |
         ((uint64_t)p[6] << 48) | ((uint64_t)p[7] << 56);
}

__host__ __device__ __forceinline__ void
u64_le_store(uint8_t* p, uint64_t w)
{
  p[0] = (uint8_t)w;
  p[1] = (uint8_t)(w >> 8);
  p[2] = (uint8_t)(w >> 16);
  p[3] = (uint8_t)(w >> 24);
  p[4] = (uint8_t)(w >> 32);
  p[5] = (uint8_t)(w >> 40);
  p[6] = (uint8_t)(w >> 48);
  p[7] = (uint8_t)(w >> 56);
}

__device__ __forceinline__ size_t
global_block_index()
{
  return ((size_t)blockIdx.z * gridDim.y + blockIdx.y) * gridDim.x + blockIdx.x;
}

inline int
make_grid(size_t num_blocks, dim3* out)
{
  if (num_blocks == 0) {
    num_blocks = 1;
  }

  int dev = 0;
  KT_TRY(cudaGetDevice(&dev), KT_ERR_NO_DEVICE);

  int max_x = 0, max_y = 0, max_z = 0;
  KT_TRY(cudaDeviceGetAttribute(&max_x, cudaDevAttrMaxGridDimX, dev), KT_ERR_NO_DEVICE);
  KT_TRY(cudaDeviceGetAttribute(&max_y, cudaDevAttrMaxGridDimY, dev), KT_ERR_NO_DEVICE);
  KT_TRY(cudaDeviceGetAttribute(&max_z, cudaDevAttrMaxGridDimZ, dev), KT_ERR_NO_DEVICE);

  const size_t mx = (size_t)max_x;
  const size_t my = (size_t)max_y;
  const size_t mz = (size_t)max_z;

  if (num_blocks <= mx) {
    *out = dim3((unsigned)num_blocks, 1, 1);
    return KT_OK;
  }

  const size_t gy = (num_blocks + mx - 1) / mx;
  if (gy <= my) {
    *out = dim3((unsigned)mx, (unsigned)gy, 1);
    return KT_OK;
  }

  const size_t plane = mx * my;
  const size_t gz = (num_blocks + plane - 1) / plane;
  if (gz > mz) {
    return KT_ERR_GRID;
  }

  *out = dim3((unsigned)mx, (unsigned)my, (unsigned)gz);
  return KT_OK;
}

} // namespace kangarootwelve::utils
