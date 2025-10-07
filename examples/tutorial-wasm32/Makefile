# WASI SDK configuration
WASI_VERSION := 27
WASI_VERSION_FULL := $(WASI_VERSION).0
WASI_ARCH := $(shell uname -m)
WASI_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]' | sed 's/darwin/macos/')
WASI_SDK_DIR := wasi-sdk-$(WASI_VERSION_FULL)-$(WASI_ARCH)-$(WASI_OS)
WASI_SDK_PATH := $(CURDIR)/$(WASI_SDK_DIR)
# e.g. https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-27/wasi-sdk-27.0-x86_64-linux.tar.gz
WASI_SDK_URL := https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$(WASI_VERSION)/wasi-sdk-$(WASI_VERSION_FULL)-$(WASI_ARCH)-$(WASI_OS).tar.gz

# WASI build flags for C++ compilation with wasmtime
WASIFLAGS = -std=c++14 \
           --target=wasm32-wasi \
           --sysroot=$(WASI_SDK_PATH)/share/wasi-sysroot \
           -D_WASI_EMULATED_SIGNAL \
           -lwasi-emulated-signal \
           -fno-exceptions

# Use the WASI SDK C++ compiler, regardless of the CXX set by the user.
CXX = $(WASI_SDK_PATH)/bin/clang++

# Default target
all: run a.out

# Run the example with wasmtime (installs all deps automatically)
run: main.wasm
	wasmtime run --allow-precompiled main.wasm

# CI expects a.out; we create it as a bash script that simply execs wasmtime
a.out: main.wasm
	@printf '%s\n' '#!/usr/bin/env bash' 'exec wasmtime run --allow-precompiled "$$(dirname "$$0")/main.wasm" "$$@"' > $@
	@chmod +x $@

main.wasm: main.cpp generated.h generated.cpp \
           target/wasm32-wasip1/release/libexample_tutorial_wasm32.a \
           wasi-sdk-installed
	$(CXX) $(WASIFLAGS) \
	    main.cpp generated.cpp \
	    target/wasm32-wasip1/release/libexample_tutorial_wasm32.a \
	    -o $@

target/wasm32-wasip1/release/libexample_tutorial_wasm32.a: wasm32-wasip1-target generated.h ./src/generated.rs ./src/lib.rs
	cargo build --target=wasm32-wasip1 --release

generated.h ./src/generated.rs generated.cpp: main32.zng
	cargo run --release --manifest-path ../../zngur-cli/Cargo.toml g main32.zng

# Ensure wasm32-wasip1 target is installed
wasm32-wasip1-target:
	@rustup target list --installed | grep -q wasm32-wasip1 || { \
		echo "Installing wasm32-wasip1 Rust target..."; \
		rustup target add wasm32-wasip1; \
	}
	@touch wasm32-wasip1-target

# Ensure WASI SDK is installed
wasi-sdk-installed: $(WASI_SDK_DIR)
	@touch wasi-sdk-installed

$(WASI_SDK_DIR):
	@echo "Downloading WASI SDK $(WASI_VERSION_FULL) for $(WASI_ARCH)-$(WASI_OS)..."
	@curl -L $(WASI_SDK_URL) -o $(WASI_SDK_DIR).tar.gz
	@echo "Extracting WASI SDK..."
	@tar -xzf $(WASI_SDK_DIR).tar.gz
	@rm $(WASI_SDK_DIR).tar.gz
	@echo "WASI SDK installed at $(WASI_SDK_PATH)"

clean:
	cargo clean
	rm -f generated.h ./src/generated.rs generated.cpp generated.o
	rm -f main.wasm example_tutorial_wasm32.wasm a.out
	rm -f wasm32-wasip1-target wasi-sdk-installed
	rm -rf wasi-sdk-*

# Manual WASI SDK installation target
install-wasi-sdk: wasi-sdk-installed

.PHONY: clean install-wasi-sdk
