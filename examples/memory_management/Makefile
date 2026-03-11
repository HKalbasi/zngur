a.out: main.cpp generated.h zngur.h src/generated.rs src/lib.rs ../../target/release/libexample_memory_management.a
	${CXX} -std=c++17 -Werror -I. main.cpp generated.cpp -g -L ../../target/release/ -l example_memory_management

../../target/release/libexample_memory_management.a:
	cargo build --release

generated.h generated.cpp ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/memory_management/main.zng

zngur.h:
	cd ../../zngur-cli && cargo run -- make-zng-header --panic-to-exception ../examples/memory_management/zngur.h

.PHONY: ../../target/release/libexample_memory_management.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
