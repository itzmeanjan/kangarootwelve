# Build & run the CUDA C++ API examples.
#
#   make            # build all examples
#   make run        # build and run the hashing example
#   make clean      # remove build artifacts
#
# Override the toolchain / target arch as needed:
#   make KT_CUDA_ARCH=sm_89           # e.g. NVIDIA L40S (Ada)
#   make NVCC=/usr/local/cuda/bin/nvcc

NVCC        ?= nvcc
KT_CUDA_ARCH ?= sm_80
NVCC_FLAGS  ?= -O3 -std=c++20 -arch=$(KT_CUDA_ARCH)

HEADERS     := kangarootwelve.cuh keccak.cuh sponge.cuh utils.cuh
EXAMPLE_DIR := examples
BUILD_DIR   := build

EXAMPLES    := kt128
BINARIES    := $(addprefix $(BUILD_DIR)/,$(EXAMPLES))

.PHONY: all run clean

all: $(BINARIES)

$(BUILD_DIR)/%: $(EXAMPLE_DIR)/%.cu $(HEADERS) | $(BUILD_DIR)
	$(NVCC) $(NVCC_FLAGS) $< -o $@

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

run: $(BUILD_DIR)/kt128
	./$(BUILD_DIR)/kt128

clean:
	rm -rf $(BUILD_DIR)
