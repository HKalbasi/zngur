a.out: generated.o impls.o src/generated.rs src/main.rs
	cargo b --release
	cp ../../target/release/example-cpp-inheritance a.out

impls.o: task.h impls.h cpp_rust_inheritance.h generated.h impls.cpp
	${CXX} -std=c++11 -c -Werror -I. impls.cpp

generated.o: task.h generated.h generated.cpp impls.h
	${CXX} -std=c++11 -c -Werror -I. generated.cpp

generated.h src/generated.rs generated.cpp: main.zng zngur.h
	cd ../../zngur-cli && cargo run g ../examples/cpp_inheritance/main.zng --crate-name "crate"

zngur.h:
	cd ../../zngur-cli && cargo run h ../examples/cpp_inheritance/zngur.h
	

.PHONY: clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt *.o
