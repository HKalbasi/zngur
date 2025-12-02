CXX ?= clang++

# Default target
all: a.out

# Step 1: Generate the bridge project (auto-builds user crate if needed for layout extraction)
zngur-bridge/: main.zng src/lib.rs Cargo.toml
	../../target/debug/zngur gs main.zng

# Step 2: Build the bridge library (cargo automatically builds user's crate as dependency)
zngur-bridge/target/release/libzngur_bridge.a: zngur-bridge/ src/lib.rs Cargo.toml
	cd zngur-bridge && cargo build --release

# Step 3: Compile and link the C++ executable (only link against bridge)
a.out: main.cpp zngur-bridge/target/release/libzngur_bridge.a zngur-bridge/cpp/generated.h
	${CXX} -std=c++17 -Werror main.cpp \
		-g \
		-I . \
		-L zngur-bridge/target/release -lzngur_bridge \
		-framework Security

.PHONY: clean test

clean:
	rm -rf zngur-bridge a.out actual_output.txt target

test: a.out
	./a.out 2>&1 | sed 's/thread.*panicked/thread panicked/g' > actual_output.txt
	diff expected_output.txt actual_output.txt
