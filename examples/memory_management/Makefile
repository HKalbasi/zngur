a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_memory_management.a
	${CXX} main.cpp -g -L ../../target/release/ -l example_memory_management

../../target/release/libexample_memory_management.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/memory_management/main.zng

FORCE: ;
