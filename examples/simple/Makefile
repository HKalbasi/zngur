a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple.a
	${CXX} -std=c++11 -Werror main.cpp generated.cpp -g -L ../../target/release/ -l example_simple

../../target/release/libexample_simple.a:
	cargo build --release

generated.h generated.cpp ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/simple/main.zng

.PHONY: ../../target/release/libexample_simple.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
