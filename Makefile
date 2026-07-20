# Collects inspiration from https://github.com/0xMiden/miden-base/blob/983357b2ad42f6e8d3c338d460a69479b99a1136/Makefile

.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

BACKTRACE=RUST_BACKTRACE=1

# -------------------------------------------------- Testing ----------------------------------------------------------

.PHONY: test
test: ## Run all tests against the default single-threaded backend
	$(BACKTRACE) RUSTFLAGS="-C target-cpu=native" cargo test --release

.PHONY: test-mt
test-mt: ## Run all tests against the multi-threaded backend
	$(BACKTRACE) RUSTFLAGS="-C target-cpu=native" cargo test --release --features multi_threaded

.PHONY: test-cuda
test-cuda: ## Run all tests against the CUDA backend
	$(BACKTRACE) cargo test --release --features cuda

.PHONY: test-wasm
test-wasm: ## Run all tests against the single-threaded backend, in WASM environment
	$(BACKTRACE) cargo test --target wasm32-wasip1 --release --no-default-features
	$(BACKTRACE) cargo test --target wasm32-wasip2 --release --no-default-features

.PHONY: coverage
coverage: ## Generates HTML code coverage report, using `cargo-tarpaulin`
	cargo tarpaulin -t 600 --release --out Html

# ----------------------------------------------- Benchmarking --------------------------------------------------------

.PHONY: bench
bench: ## Run all benchmarks against the default single-threaded backend
	RUSTFLAGS="-C target-cpu=native" cargo criterion

.PHONY: bench-mt
bench-mt: ## Run all benchmarks against the multi-threaded backend
	RUSTFLAGS="-C target-cpu=native" cargo criterion --features multi_threaded

.PHONY: bench-cuda
bench-cuda: ## Run all benchmarks against the CUDA backend
	RUSTFLAGS="-C target-cpu=native" cargo criterion --features cuda

# ------------------------------------------- Using the library -------------------------------------------------------

.PHONY: example
example: ## Run examples, single-threaded
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt128
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt256

.PHONY: example-mt
example-mt: ## Run examples, multi-threaded
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt128 --features multi_threaded
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt256 --features multi_threaded

.PHONY: example-cuda
example-cuda: ## Run examples, on NVIDIA GPU
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt128 --features cuda
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt256 --features cuda
	RUSTFLAGS="-C target-cpu=native" cargo run --example kt128_cuda --features cuda

.PHONY: example-wasm
example-wasm: ## Run examples in WASM environment
	cargo run --example kt128 --target wasm32-wasip1 --no-default-features
	cargo run --example kt256 --target wasm32-wasip1 --no-default-features
	cargo run --example kt128 --target wasm32-wasip2 --no-default-features
	cargo run --example kt256 --target wasm32-wasip2 --no-default-features

# --------------------------------- Developing and maintaining the library --------------------------------------------

.PHONY: clippy
clippy: ## Shows warnings
	cargo clippy --all --all-targets -- -D warnings
	cargo clippy --all --all-targets --features multi_threaded -- -D warnings
	cargo clippy --all --all-targets --features cuda -- -D warnings

.PHONY: format
format: ## Formats source files
	cargo fmt --all

.PHONY: clean
clean: ## Removes build directory
	cargo clean
