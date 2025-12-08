a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_raw_pointer.a
	${CXX} --std=c++11 -Werror -I. main.cpp -g -L ../../target/release/ -l example_raw_pointer

../../target/release/libexample_raw_pointer.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/raw_pointer/main.zng -o ../examples/raw_pointer

.PHONY: ../../target/release/libexample_raw_pointer.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
