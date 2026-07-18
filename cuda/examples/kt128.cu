/**
 * Example: hashing a message with the KT128 CUDA C++ API
 *
 * Build & run:  `make run` from the `cuda/` directory.
 */

#include <array>
#include <cstdint>
#include <cstdio>
#include <span>
#include <string>
#include <vector>

#include <cuda_runtime.h>

#include "../kangarootwelve.cuh"

using namespace kangarootwelve;
constexpr size_t DIGEST_BYTE_LEN = 32;

int
main()
{
  // The message and (optional) customization string to hash with KT128.
  const std::string text = "The quick brown fox jumps over the lazy dog";
  const std::vector<uint8_t> msg(text.begin(), text.end());
  const std::vector<uint8_t> cstr = {};

  // 1) Upload the message to device memory.
  uint8_t* dmsg = nullptr;
  if (cudaMalloc((void**)&dmsg, msg.size()) != cudaSuccess) {
    fprintf(stderr, "cudaMalloc failed\n");
    return 1;
  }
  if (!msg.empty() && cudaMemcpy(dmsg, msg.data(), msg.size(), cudaMemcpyHostToDevice) != cudaSuccess) {
    fprintf(stderr, "cudaMemcpy H2D failed\n");
    cudaFree(dmsg);
    return 1;
  }

  // 2) Absorb on the GPU and get back final sponge state.
  std::array<uint8_t, KECCAK_PERMUTATION_BYTE_WIDTH> out_state = {};
  uint64_t elapsed_ns = 0;

  const int rc = hash_device<KT128_RATE_BYTES, KT128_CV_BYTES>(
    dmsg, msg.size(), std::span<const uint8_t>(cstr), std::span<uint8_t, KECCAK_PERMUTATION_BYTE_WIDTH>(out_state), &elapsed_ns);
  cudaFree(dmsg);

  if (rc != KT_OK) {
    fprintf(stderr, "hash_device failed with status %d\n", rc);
    return 1;
  }

  // 3) Squeeze a 32-byte digest out of the absorbed KT128 state, on the host.
  uint64_t state[keccak::LANE_COUNT];
  for (size_t k = 0; k < keccak::LANE_COUNT; k++) {
    state[k] = utils::u64_le_load(out_state.data() + k * sizeof(uint64_t));
  }

  uint8_t digest[DIGEST_BYTE_LEN] = {};
  size_t offset = 0;
  sponge::squeeze(state, &offset, digest, DIGEST_BYTE_LEN, KT128_RATE_BYTES);

  printf("message : \"%s\"\n", text.c_str());
  printf("KT128   : ");
  for (size_t i = 0; i < DIGEST_BYTE_LEN; i++) {
    printf("%02x", digest[i]);
  }
  printf("\n");
  printf("gpu absorb time : %.3f us\n", (double)elapsed_ns / 1000.0);

  return 0;
}
