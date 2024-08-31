ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
RUST_LOG ?= info

RUST_FLAGS := --release
RUST_TARGET := release

.PHONY: index
index: INPUT ?= $(ROOT_DIR)data/raw/rockyou.csv
index: OUTPUT ?= $(ROOT_DIR)data/index
index: THREADS ?= 8
index:
	cargo run $(RUST_FLAGS) --bin index -- --input=$(INPUT) --output=$(OUTPUT) --threads=$(THREADS)

.PHONY: build
build:
	cd crates/parse-ffi && RUST_LOG=$(RUST_LOG) cargo build $(RUST_FLAGS)
	cp crates/parse-ffi/target/$(RUST_TARGET)/libparse_ffi.dylib lib/ \
	 || cp crates/parse-ffi/target/$(RUST_TARGET)/libparse_ffi.so lib/
	go build -ldflags="-r $(ROOT_DIR)lib" .

run: build
	RUST_LOG=$(RUST_LOG) go run .

debug: RUST_LOG := debug
debug: RUST_FLAGS :=
debug: RUST_TARGET := debug
debug: build run
