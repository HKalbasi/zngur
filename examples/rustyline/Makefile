a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_rustyline.a
	clang++ main.cpp -g -L ../../target/release/ -l example_rustyline

../../target/release/libexample_rustyline.a: FORCE
	cargo build --release

generated.h: main.zng
	cd ../../zngur && cargo run g ../examples/rustyline/main.zng

FORCE: ;
