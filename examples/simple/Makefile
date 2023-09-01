a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple.a
	clang++ main.cpp -g -L ../../target/release/ -l example_simple

../../target/release/libexample_simple.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/simple/main.zng

FORCE: ;
