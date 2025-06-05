a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple_import.a
	${CXX} -Werror main.cpp generated.cpp -g -L ../../target/release/ -l example_simple_import

../../target/release/libexample_simple_import.a: FORCE
	cargo build --release

generated.h generated.cpp ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/simple_import/main.zng

FORCE: ;
