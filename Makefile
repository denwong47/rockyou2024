ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

.PHONY: build-dynamic
build:
	@cd crates/parse-ffi && cargo build --release
	@cp crates/parse-ffi/target/release/libparse_ffi.dylib lib/ \
	 || cp crates/parse-ffi/target/release/libparse_ffi.so lib/
	go build -ldflags="-r $(ROOT_DIR)lib" .

run: build
	go run .
