a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_raw_pointer.a
	${CXX} -Werror main.cpp -g -L ../../target/release/ -l example_raw_pointer

../../target/release/libexample_raw_pointer.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/raw_pointer/main.zng

FORCE: ;
