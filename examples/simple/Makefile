a.out: main.cpp generated.h zngur.h src/generated.rs src/lib.rs ../../target/release/libexample_simple.a
	${CXX} -std=c++17 -Werror -I. main.cpp generated.cpp -g -L ../../target/release/ -l example_simple

../../target/release/libexample_simple.a:
	cargo build --release

generated.h generated.cpp ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/simple/main.zng

zngur.h:
	cd ../../zngur-cli && cargo run -- make-zng-header --panic-to-exception ../examples/simple/zngur.h

.PHONY: ../../target/release/libexample_simple.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
