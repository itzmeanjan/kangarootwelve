/**
 * KangarooTwelve family of hashing on NVIDIA GPUs, in CUDA C++.
 * Specification: https://www.rfc-editor.org/rfc/rfc9861.html
 */

#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <cuda_runtime.h>
#include <vector>

#include "keccak.cuh"
#include "sponge.cuh"
#include "utils.cuh"

namespace kangarootwelve {

constexpr size_t CHUNK_BYTE_LEN = 8192;
constexpr unsigned THREADS_PER_BLOCK = 64;

constexpr size_t LEAVES_PER_BATCH = 8192;
constexpr int PIPELINE_DEPTH = 3;

constexpr uint8_t D_SEP_SINGLE = 0x07;
constexpr uint8_t D_SEP_LEAF = 0x0b;
constexpr uint8_t D_SEP_FINAL = 0x06;

constexpr size_t KT128_TARGET_BIT_SECURITY = 128;
constexpr size_t KT128_CAPACITY_BITS = 2 * KT128_TARGET_BIT_SECURITY;
constexpr size_t KT128_RATE_BITS = 1600 - KT128_CAPACITY_BITS;
constexpr size_t KT128_RATE_BYTES = KT128_RATE_BITS / 8;
constexpr size_t KT128_CV_BYTES = KT128_CAPACITY_BITS / 8;

constexpr size_t KT256_TARGET_BIT_SECURITY = 256;
constexpr size_t KT256_CAPACITY_BITS = 2 * KT256_TARGET_BIT_SECURITY;
constexpr size_t KT256_RATE_BITS = 1600 - KT256_CAPACITY_BITS;
constexpr size_t KT256_RATE_BYTES = KT256_RATE_BITS / 8;
constexpr size_t KT256_CV_BYTES = KT256_CAPACITY_BITS / 8;

inline size_t
length_encode_host(uint64_t x, uint8_t res[9])
{
  int bw = 0;
  uint64_t t = x;

  while (t) {
    bw++;
    t >>= 1;
  }

  int l = (bw + 7) / 8;
  for (int i = 0; i < l; i++) {
    res[l - 1 - i] = (uint8_t)(x >> (8 * i));
  }

  res[l] = (uint8_t)l;
  return (size_t)l + 1;
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

template<size_t RATE, size_t CVLEN>
int
hash(const uint8_t* msg, size_t mlen, const uint8_t* cstr, size_t clen, uint8_t out_state[keccak::LANE_COUNT * 8])
{
  uint8_t enc[9];
  const size_t elen = length_encode_host(clen, enc);
  const size_t tlen = mlen + clen + elen;

  std::vector<uint8_t> S(tlen);
  if (mlen) {
    memcpy(S.data(), msg, mlen);
  }
  if (clen) {
    memcpy(S.data() + mlen, cstr, clen);
  }
  memcpy(S.data() + mlen + clen, enc, elen);

  const size_t n = (tlen + CHUNK_BYTE_LEN - 1) / CHUNK_BYTE_LEN;

  if (n == 1) {
    uint8_t* dS = nullptr;
    uint8_t* dState = nullptr;

    const int status = [&]() -> int {
      KT_TRY(cudaMalloc(&dS, tlen), KT_ERR_ALLOC);
      KT_TRY(cudaMalloc(&dState, keccak::LANE_COUNT * 8), KT_ERR_ALLOC);
      KT_TRY(cudaMemcpy(dS, S.data(), tlen, cudaMemcpyHostToDevice), KT_ERR_MEMCPY);

      single_node_kernel<RATE><<<1, 1>>>(dS, tlen, dState);
      KT_TRY(cudaGetLastError(), KT_ERR_KERNEL);
      KT_TRY(cudaMemcpy(out_state, dState, keccak::LANE_COUNT * 8, cudaMemcpyDeviceToHost), KT_ERR_MEMCPY);

      return KT_OK;
    }();

    if (dState) {
      cudaFree(dState);
    }
    if (dS) {
      cudaFree(dS);
    }

    return status;
  }

  const size_t leaves = n - 1;
  const size_t nb = (leaves + LEAVES_PER_BATCH - 1) / LEAVES_PER_BATCH;
  const int depth = (nb < (size_t)PIPELINE_DEPTH) ? (int)nb : PIPELINE_DEPTH;

  uint8_t* dS = nullptr;
  uint8_t* dCV = nullptr;
  std::vector<cudaStream_t> streams;
  std::vector<cudaEvent_t> events;
  std::vector<uint8_t*> hbuf;

  const int status = [&]() -> int {
    KT_TRY(cudaMalloc(&dS, tlen), KT_ERR_ALLOC);
    KT_TRY(cudaMemcpy(dS, S.data(), tlen, cudaMemcpyHostToDevice), KT_ERR_MEMCPY);
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

      const size_t num_blocks = (count + THREADS_PER_BLOCK - 1) / THREADS_PER_BLOCK;
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

    uint64_t state[keccak::LANE_COUNT] = {};
    size_t offset = 0;

    sponge::absorb(state, &offset, S.data(), CHUNK_BYTE_LEN, RATE);
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

    uint8_t enc_n[9];
    const size_t elen_n = length_encode_host(leaves, enc_n);

    sponge::absorb(state, &offset, enc_n, elen_n, RATE);
    sponge::absorb(state, &offset, padB, 2, RATE);
    sponge::finalize(state, &offset, RATE, D_SEP_FINAL);

    for (size_t k = 0; k < keccak::LANE_COUNT; k++) {
      utils::u64_le_store(out_state + k * 8, state[k]);
    }

    return KT_OK;
  }();

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
  if (dS) {
    cudaFree(dS);
  }

  return status;
}

} // namespace kangarootwelve
