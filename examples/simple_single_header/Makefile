a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple_single_header.a
	${CXX} -std=c++11 -Werror main.cpp generated.cpp -g -L ../../target/release/ -l example_simple_single_header

../../target/release/libexample_simple_single_header.a:
	cargo build --release

generated.h generated.cpp ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/simple_single_header/main.zng --single-header

.PHONY: ../../target/release/libexample_simple_single_header.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
