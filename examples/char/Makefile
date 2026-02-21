a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_char.a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_char

../../target/release/libexample_char.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/char/main.zng

.PHONY: ../../target/release/libexample_char.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
