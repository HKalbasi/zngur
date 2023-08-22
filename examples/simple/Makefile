a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple.a
	clang++ main.cpp -g -L ../../target/release/ -l example_simple

../../target/release/libexample_simple.a:
	cargo build --release

generated.h: main.zng
	cd ../../zngur && cargo run g ../examples/simple/main.zng

