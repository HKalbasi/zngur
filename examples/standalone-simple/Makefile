CXX ?= clang++

# Default target
all: a.out

# Step 1: Generate the bridge project template (no compilation yet)
zngur-bridge/: main.zng
	../../target/debug/zngur gs main.zng

# Step 2: Build the bridge library (build.rs does layout extraction and code generation)
# This produces:
#   - zngur-bridge/target/release/libzngur_bridge.a (Rust staticlib)
#   - zngur-bridge/cpp/generated.h and generated.cpp (for host app to compile)
zngur-bridge/target/release/libzngur_bridge.a zngur-bridge/cpp/generated.cpp: zngur-bridge/ src/lib.rs Cargo.toml
	cargo build --manifest-path zngur-bridge/Cargo.toml --release

# Step 3: Compile C++ (host app compiles generated.cpp alongside main.cpp for ABI compatibility)
a.out: main.cpp zngur-bridge/target/release/libzngur_bridge.a zngur-bridge/cpp/generated.cpp
	${CXX} -std=c++17 -Werror main.cpp zngur-bridge/cpp/generated.cpp \
		-g \
		-I zngur-bridge/cpp \
		-L zngur-bridge/target/release -lzngur_bridge \
		-framework Security

.PHONY: clean test

clean:
	rm -rf zngur-bridge a.out actual_output.txt target

test: a.out
	./a.out 2>&1 | sed 's/thread.*panicked/thread panicked/g' > actual_output.txt
	diff expected_output.txt actual_output.txt
