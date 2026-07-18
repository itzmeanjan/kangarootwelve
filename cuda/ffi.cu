#include <cstddef>
#include <cstdint>
#include <new>
#include <span>

#include "kangarootwelve.cuh"

using namespace kangarootwelve;

extern "C" int
kt128_cuda_absorb_device(const uint8_t* dmsg,
                         size_t mlen,
                         const uint8_t* cstr,
                         size_t clen,
                         uint8_t out_state[KECCAK_PERMUTATION_BYTE_WIDTH],
                         uint64_t* out_elapsed_ns)
{
  try {
    return hash_device<KT128_RATE_BYTES, KT128_CV_BYTES>(dmsg,
                                                         mlen,
                                                         std::span<const uint8_t>(cstr, clen),
                                                         std::span<uint8_t, KECCAK_PERMUTATION_BYTE_WIDTH>(out_state, KECCAK_PERMUTATION_BYTE_WIDTH),
                                                         out_elapsed_ns);
  } catch (const std::bad_alloc&) {
    return KT_ERR_ALLOC;
  }
}

extern "C" int
kt256_cuda_absorb_device(const uint8_t* dmsg,
                         size_t mlen,
                         const uint8_t* cstr,
                         size_t clen,
                         uint8_t out_state[KECCAK_PERMUTATION_BYTE_WIDTH],
                         uint64_t* out_elapsed_ns)
{
  try {
    return hash_device<KT256_RATE_BYTES, KT256_CV_BYTES>(dmsg,
                                                         mlen,
                                                         std::span<const uint8_t>(cstr, clen),
                                                         std::span<uint8_t, KECCAK_PERMUTATION_BYTE_WIDTH>(out_state, KECCAK_PERMUTATION_BYTE_WIDTH),
                                                         out_elapsed_ns);
  } catch (const std::bad_alloc&) {
    return KT_ERR_ALLOC;
  }
}

extern "C" uint8_t*
kt_cuda_upload(const uint8_t* src, size_t len)
{
  uint8_t* dptr = nullptr;
  if (cudaMalloc((void**)&dptr, len) != cudaSuccess) {
    return nullptr;
  }
  if (len && cudaMemcpy(dptr, src, len, cudaMemcpyHostToDevice) != cudaSuccess) {
    cudaFree(dptr);
    return nullptr;
  }

  return dptr;
}

extern "C" void
kt_cuda_release(uint8_t* dptr)
{
  if (dptr) {
    cudaFree(dptr);
  }
}
