a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_rayon.a
	clang++ main.cpp -g -L ../../target/release/ -l example_rayon

../../target/release/libexample_rayon.a: FORCE
	cargo build --release

generated.h: main.zng
	cd ../../zngur && cargo run g ../examples/rayon/main.zng

FORCE: ;
