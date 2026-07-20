/**
 * KangarooTwelve family of hashing on NVIDIA GPUs, in CUDA C++.
 * Specification: https://www.rfc-editor.org/rfc/rfc9861.html
 */

#pragma once

#include <array>
#include <bit>
#include <cstddef>
#include <cstdint>
#include <cuda_runtime.h>
#include <limits>
#include <span>
#include <utility>
#include <vector>

#include "keccak.cuh"
#include "sponge.cuh"
#include "utils.cuh"

namespace kangarootwelve {

constexpr size_t KECCAK_PERMUTATION_BIT_WIDTH = 1600;
constexpr size_t KECCAK_PERMUTATION_BYTE_WIDTH = KECCAK_PERMUTATION_BIT_WIDTH / std::numeric_limits<uint8_t>::digits;
constexpr size_t CHUNK_BYTE_LEN = 8192;
constexpr size_t LENGTH_ENCODE_MAX_BYTE_LEN = sizeof(uint64_t) + 1;

constexpr unsigned THREADS_PER_BLOCK = 128;
constexpr size_t LEAVES_PER_BATCH = 32768;
constexpr size_t PIPELINE_DEPTH = 3;

static_assert(THREADS_PER_BLOCK > 0, "Must be non-zero");
static_assert(THREADS_PER_BLOCK <= 1024, "Must not exceed the CUDA hardware limit of 1024 threads per block");
static_assert(LEAVES_PER_BATCH > 0, "Must be non-zero");
static_assert(PIPELINE_DEPTH > 0, "Must be non-zero");

constexpr uint8_t D_SEP_SINGLE = 0x07;
constexpr uint8_t D_SEP_LEAF = 0x0b;
constexpr uint8_t D_SEP_FINAL = 0x06;

constexpr size_t KT128_TARGET_BIT_SECURITY = 128;
constexpr size_t KT128_CAPACITY_BITS = 2 * KT128_TARGET_BIT_SECURITY;
constexpr size_t KT128_RATE_BITS = KECCAK_PERMUTATION_BIT_WIDTH - KT128_CAPACITY_BITS;
constexpr size_t KT128_RATE_BYTES = KT128_RATE_BITS / std::numeric_limits<uint8_t>::digits;
constexpr size_t KT128_CV_BYTES = KT128_CAPACITY_BITS / std::numeric_limits<uint8_t>::digits;

constexpr size_t KT256_TARGET_BIT_SECURITY = 256;
constexpr size_t KT256_CAPACITY_BITS = 2 * KT256_TARGET_BIT_SECURITY;
constexpr size_t KT256_RATE_BITS = KECCAK_PERMUTATION_BIT_WIDTH - KT256_CAPACITY_BITS;
constexpr size_t KT256_RATE_BYTES = KT256_RATE_BITS / std::numeric_limits<uint8_t>::digits;
constexpr size_t KT256_CV_BYTES = KT256_CAPACITY_BITS / std::numeric_limits<uint8_t>::digits;

inline std::pair<std::array<uint8_t, LENGTH_ENCODE_MAX_BYTE_LEN>, size_t>
length_encode(uint64_t x)
{
  std::array<uint8_t, LENGTH_ENCODE_MAX_BYTE_LEN> res = {};
  const unsigned l = utils::ceil_div<unsigned>(std::bit_width(x), std::numeric_limits<uint8_t>::digits);

  for (unsigned i = 0; i < l; i++) {
    res[l - 1 - i] = (uint8_t)(x >> (std::numeric_limits<uint8_t>::digits * i));
  }

  res[l] = (uint8_t)l;
  return { res, (size_t)l + 1 };
}

template<size_t RATE>
__global__ void
single_node_kernel(const uint8_t* S, size_t tlen, uint8_t* state_out)
{
  uint64_t state[keccak::LANE_COUNT] = {};
  size_t offset = 0;

  sponge::absorb(state, &offset, S, tlen, RATE);
  sponge::finalize(state, &offset, RATE, D_SEP_SINGLE);

  for (size_t k = 0; k < keccak::LANE_COUNT; k++) {
    utils::u64_le_store(state_out + k * 8, state[k]);
  }
}

template<size_t RATE, size_t CVLEN>
__global__ void
leaf_kernel(const uint8_t* S, size_t tlen, size_t leaf_begin, size_t count, uint8_t* cv_out)
{
  const size_t i = utils::global_block_index() * blockDim.x + threadIdx.x;
  if (i >= count) {
    return;
  }

  const size_t t = leaf_begin + i;
  const size_t off_in_S = (1 + t) * CHUNK_BYTE_LEN;
  size_t clen = tlen - off_in_S;

  if (clen > CHUNK_BYTE_LEN) {
    clen = CHUNK_BYTE_LEN;
  }

  uint64_t state[keccak::LANE_COUNT] = {};
  size_t offset = 0;

  sponge::absorb(state, &offset, S + off_in_S, clen, RATE);
  sponge::finalize(state, &offset, RATE, D_SEP_LEAF);

  static_assert(CVLEN % 8 == 0, "CVLEN must be a multiple of 8");
  uint8_t* cv = cv_out + i * CVLEN;

  for (size_t k = 0; k < CVLEN / 8; k++) {
    utils::u64_le_store(cv + k * 8, state[k]);
  }
}

inline int
assemble_S_from_device(uint8_t* dS, const uint8_t* dmsg, size_t mlen, std::span<const uint8_t> cstr, std::span<const uint8_t> enc)
{
  if (mlen) {
    KT_TRY(cudaMemcpy(dS, dmsg, mlen, cudaMemcpyDeviceToDevice), KT_ERR_MEMCPY);
  }
  if (!cstr.empty()) {
    KT_TRY(cudaMemcpy(dS + mlen, cstr.data(), cstr.size(), cudaMemcpyHostToDevice), KT_ERR_MEMCPY);
  }
  KT_TRY(cudaMemcpy(dS + mlen + cstr.size(), enc.data(), enc.size(), cudaMemcpyHostToDevice), KT_ERR_MEMCPY);

  return KT_OK;
}

template<size_t RATE, size_t CVLEN>
int
hash_device_S(const uint8_t* dS, size_t tlen, std::span<uint8_t, KECCAK_PERMUTATION_BYTE_WIDTH> out_state, uint64_t* out_elapsed_ns)
{
  const size_t n = utils::ceil_div(tlen, CHUNK_BYTE_LEN);

  uint64_t elapsed_ns = 0;

  if (n == 1) {
    uint8_t* dState = nullptr;

    const int status = [&]() -> int {
      KT_TRY(cudaMalloc(&dState, KECCAK_PERMUTATION_BYTE_WIDTH), KT_ERR_ALLOC);

      const utils::compute_timer timer;

      single_node_kernel<RATE><<<1, 1>>>(dS, tlen, dState);
      KT_TRY(cudaGetLastError(), KT_ERR_KERNEL);
      KT_TRY(cudaMemcpy(out_state.data(), dState, KECCAK_PERMUTATION_BYTE_WIDTH, cudaMemcpyDeviceToHost), KT_ERR_MEMCPY);

      elapsed_ns = timer.stop();
      return KT_OK;
    }();

    if (out_elapsed_ns) {
      *out_elapsed_ns = elapsed_ns;
    }

    if (dState) {
      cudaFree(dState);
    }

    return status;
  }

  const size_t leaves = n - 1;
  const size_t nb = utils::ceil_div(leaves, LEAVES_PER_BATCH);
  const size_t depth = (nb < PIPELINE_DEPTH) ? nb : PIPELINE_DEPTH;

  uint8_t head_buf[CHUNK_BYTE_LEN] = { 0 };

  uint8_t* dCV = nullptr;

  std::vector<cudaStream_t> streams;
  std::vector<cudaEvent_t> events;
  std::vector<uint8_t*> hbuf;

  const int status = [&]() -> int {
    KT_TRY(cudaMemcpy(head_buf, dS, CHUNK_BYTE_LEN, cudaMemcpyDeviceToHost), KT_ERR_MEMCPY);
    KT_TRY(cudaMalloc(&dCV, leaves * CVLEN), KT_ERR_ALLOC);

    for (int s = 0; s < depth; s++) {
      cudaStream_t stream;
      KT_TRY(cudaStreamCreate(&stream), KT_ERR_ALLOC);
      streams.push_back(stream);

      cudaEvent_t event;
      KT_TRY(cudaEventCreate(&event), KT_ERR_ALLOC);
      events.push_back(event);

      uint8_t* hb = nullptr;
      KT_TRY(cudaHostAlloc((void**)&hb, LEAVES_PER_BATCH * CVLEN, cudaHostAllocDefault), KT_ERR_ALLOC);
      hbuf.push_back(hb);
    }

    const auto batch_count = [&](size_t b) {
      const size_t begin = b * LEAVES_PER_BATCH;
      const size_t rem = leaves - begin;

      return (rem < LEAVES_PER_BATCH) ? rem : LEAVES_PER_BATCH;
    };

    const auto launch_batch = [&](size_t b) -> int {
      const int s = (int)(b % depth);
      const size_t begin = b * LEAVES_PER_BATCH;
      const size_t count = batch_count(b);

      const size_t num_blocks = utils::ceil_div(count, (size_t)THREADS_PER_BLOCK);
      dim3 grid;
      const int rc = utils::make_grid(num_blocks, &grid);
      if (rc != KT_OK) {
        return rc;
      }

      leaf_kernel<RATE, CVLEN><<<grid, THREADS_PER_BLOCK, 0, streams[s]>>>(dS, tlen, begin, count, dCV + begin * CVLEN);
      KT_TRY(cudaGetLastError(), KT_ERR_KERNEL);
      KT_TRY(cudaMemcpyAsync(hbuf[s], dCV + begin * CVLEN, count * CVLEN, cudaMemcpyDeviceToHost, streams[s]), KT_ERR_MEMCPY);
      KT_TRY(cudaEventRecord(events[s], streams[s]), KT_ERR_KERNEL);

      return KT_OK;
    };

    constexpr uint8_t padA[8] = { 3, 0, 0, 0, 0, 0, 0, 0 };
    constexpr uint8_t padB[2] = { 0xff, 0xff };

    const utils::compute_timer timer;

    uint64_t state[keccak::LANE_COUNT] = {};
    size_t offset = 0;

    sponge::absorb(state, &offset, head_buf, CHUNK_BYTE_LEN, RATE);
    sponge::absorb(state, &offset, padA, 8, RATE);

    for (int s = 0; s < depth; s++) {
      const int rc = launch_batch((size_t)s);
      if (rc != KT_OK) {
        return rc;
      }
    }

    for (size_t b = 0; b < nb; b++) {
      const int s = (int)(b % depth);
      KT_TRY(cudaEventSynchronize(events[s]), KT_ERR_SYNC);

      sponge::absorb(state, &offset, hbuf[s], batch_count(b) * CVLEN, RATE);

      const size_t next = b + depth;
      if (next < nb) {
        const int rc = launch_batch(next);
        if (rc != KT_OK) {
          return rc;
        }
      }
    }

    const auto [enc_n, elen_n] = length_encode(leaves);

    sponge::absorb(state, &offset, enc_n.data(), elen_n, RATE);
    sponge::absorb(state, &offset, padB, 2, RATE);
    sponge::finalize(state, &offset, RATE, D_SEP_FINAL);

    for (size_t k = 0; k < keccak::LANE_COUNT; k++) {
      utils::u64_le_store(out_state.data() + k * 8, state[k]);
    }

    elapsed_ns = timer.stop();
    return KT_OK;
  }();

  if (out_elapsed_ns) {
    *out_elapsed_ns = elapsed_ns;
  }

  for (uint8_t* h : hbuf) {
    cudaFreeHost(h);
  }
  for (cudaEvent_t e : events) {
    cudaEventDestroy(e);
  }
  for (cudaStream_t s : streams) {
    cudaStreamDestroy(s);
  }
  if (dCV) {
    cudaFree(dCV);
  }

  return status;
}

template<size_t RATE, size_t CVLEN>
int
hash_device(const uint8_t* dmsg,
            size_t mlen,
            std::span<const uint8_t> cstr,
            std::span<uint8_t, KECCAK_PERMUTATION_BYTE_WIDTH> out_state,
            uint64_t* out_elapsed_ns = nullptr)
{
  const auto [enc, elen] = length_encode(cstr.size());
  const size_t tlen = mlen + cstr.size() + elen;

  uint8_t* dS = nullptr;

  const int status = [&]() -> int {
    KT_TRY(cudaMalloc(&dS, tlen), KT_ERR_ALLOC);
    {
      const int rc = assemble_S_from_device(dS, dmsg, mlen, cstr, std::span<const uint8_t>(enc.data(), elen));
      if (rc != KT_OK) {
        return rc;
      }
    }

    return hash_device_S<RATE, CVLEN>(dS, tlen, out_state, out_elapsed_ns);
  }();

  if (dS) {
    cudaFree(dS);
  }

  return status;
}

} // namespace kangarootwelve
