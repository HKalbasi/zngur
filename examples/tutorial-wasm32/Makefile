# Detect platform for WASI SDK
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    WASI_SDK_PLATFORM = x86_64-linux
else ifeq ($(UNAME_S),Darwin)
    WASI_SDK_PLATFORM = x86_64-macos
else
    $(error Unsupported platform: $(UNAME_S))
endif

# WASI SDK paths and URLs
WASI_SDK_VERSION = 25.0
WASI_SDK_DIR = wasi-sdk-$(WASI_SDK_VERSION)-$(WASI_SDK_PLATFORM)
WASI_SDK_PATH = $(shell pwd)/$(WASI_SDK_DIR)
WASI_SDK_URL = $(or $(WASI_SDK_DOWNLOAD_URL),https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-25/$(WASI_SDK_DIR).tar.gz)

# WASI build flags for C++ compilation with wasmtime
WASIFLAGS = -std=c++14 \
           --target=wasm32-wasi \
           --sysroot=$(WASI_SDK_PATH)/share/wasi-sysroot \
           -D_WASI_EMULATED_SIGNAL \
           -lwasi-emulated-signal \
           -fno-exceptions

# Use the WASI SDK C++ compiler
CXX = $(WASI_SDK_PATH)/bin/clang++

# Detect wasmtime location (system PATH or home directory)
WASMTIME := $(shell which wasmtime 2>/dev/null || echo "$$HOME/.wasmtime/bin/wasmtime")

# Default target
all: run a.out

# Run the example with wasmtime (installs all deps automatically)
run: main.wasm
	$(WASMTIME) run --allow-precompiled main.wasm

# CI expects a.out; we create it as a bash script that simply execs wasmtime
a.out: main.wasm
	@printf '%s\n' '#!/usr/bin/env bash' 'exec $(WASMTIME) run --allow-precompiled "$$(dirname "$$0")/main.wasm" "$$@"' > $@
	@chmod +x $@

main.wasm: wasi-sdk-installed main.cpp generated.h generated.cpp \
           target/wasm32-wasip1/release/libexample_tutorial_wasm32.a
	$(CXX) $(WASIFLAGS) \
	    main.cpp generated.cpp \
	    target/wasm32-wasip1/release/libexample_tutorial_wasm32.a \
	    -o $@

target/wasm32-wasip1/release/libexample_tutorial_wasm32.a: wasm32-wasip1-target generated.h ./src/generated.rs ./src/lib.rs
	cargo build --target=wasm32-wasip1 --release

generated.h ./src/generated.rs generated.cpp: main32.zng
	cargo run --release --manifest-path ../../zngur-cli/Cargo.toml g main32.zng

# Install wasmtime and WASI SDK automatically
wasi-sdk-installed:
	@echo "Checking dependencies..."
	@if ! command -v wasmtime >/dev/null 2>&1 && [ ! -f "$$HOME/.wasmtime/bin/wasmtime" ]; then \
		echo "Installing wasmtime..."; \
		curl https://wasmtime.dev/install.sh -sSf | bash; \
	fi
	@if [ ! -d "$(WASI_SDK_DIR)" ]; then \
		echo "Installing WASI SDK for $(WASI_SDK_PLATFORM)..."; \
		curl -L -o wasi-sdk-latest.tar.gz "$(WASI_SDK_URL)" && \
		tar -xzf wasi-sdk-latest.tar.gz; \
	fi
	@touch wasi-sdk-installed

# Ensure wasm32-wasip1 target is installed
wasm32-wasip1-target:
	@rustup target list --installed | grep -q wasm32-wasip1 || { \
		echo "Installing wasm32-wasip1 Rust target..."; \
		rustup target add wasm32-wasip1; \
	}
	@touch wasm32-wasip1-target

clean:
	cargo clean
	rm -f generated.h ./src/generated.rs generated.cpp generated.o
	rm -f main.wasm example_tutorial_wasm32.wasm a.out wasi-sdk-latest.tar.gz
	rm -f wasi-sdk-installed wasm32-wasip1-target
	rm -rf wasi-sdk-*

# Legacy target for manual dependency installation (now automatic)
install-deps: wasi-sdk-installed wasm32-wasip1-target

.PHONY: clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
