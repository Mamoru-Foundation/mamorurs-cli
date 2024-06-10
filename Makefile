OS=$(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(shell uname -m)
BINARY_DIR = ./bin/$(OS)-$(ARCH)
BIN_NAME = mamorurs-cli

dev: build-rust  test

build-rust:
	cargo build
	mkdir -p $(BINARY_DIR)
	cp target/debug/$(BIN_NAME) $(BINARY_DIR)/

build-rust-release:
	cargo build --release
	mkdir -p $(BINARY_DIR)
	cp target/release/$(BIN_NAME) $(BINARY_DIR)/

build-rust-release-macos-aarch64:
	cargo build --release --target aarch64-apple-darwin
	mkdir -p ./bin/darwin-arm64/
	cp target/aarch64-apple-darwin/release/$(BIN_NAME) ./bin/darwin-arm64/

test:
	cargo test --workspace

lint:
	cargo fmt --all --check
	cargo clippy --workspace --tests


clean:
	cargo clean
	rm -rf generated_bindings/*

