a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_memory_management.a
	${CXX} -Werror main.cpp generated.cpp -g -L ../../target/release/ -l example_memory_management

../../target/release/libexample_memory_management.a: FORCE
	cargo build --release

generated.h generated.cpp ./src/generated.rs: main.zng FORCE
	cd ../../zngur-cli && cargo run g ../examples/memory_management/main.zng

FORCE: ;

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
