#include <cstddef>
#include <cstdint>
#include <new>

#include "kangarootwelve.cuh"

using namespace kangarootwelve;

extern "C" int
kt128_cuda_absorb(const uint8_t* msg, size_t mlen, const uint8_t* cstr, size_t clen, uint8_t out_state[keccak::LANE_COUNT * 8], uint64_t* out_elapsed_ns)
{
  try {
    return hash<KT128_RATE_BYTES, KT128_CV_BYTES>(msg, mlen, cstr, clen, out_state, out_elapsed_ns);
  } catch (const std::bad_alloc&) {
    return KT_ERR_ALLOC;
  }
}

extern "C" int
kt256_cuda_absorb(const uint8_t* msg, size_t mlen, const uint8_t* cstr, size_t clen, uint8_t out_state[keccak::LANE_COUNT * 8], uint64_t* out_elapsed_ns)
{
  try {
    return hash<KT256_RATE_BYTES, KT256_CV_BYTES>(msg, mlen, cstr, clen, out_state, out_elapsed_ns);
  } catch (const std::bad_alloc&) {
    return KT_ERR_ALLOC;
  }
}
