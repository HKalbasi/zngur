ZNGUR ?= ../../target/debug/zngur

.PHONY: ../../target/release/libexample_simple_import.a

a.out: main.cpp foo.cpp bar.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple_import.a
	${CXX} -std=c++11 main.cpp foo.cpp bar.cpp -g -L ../../target/release/ -l example_simple_import

../../target/release/libexample_simple_import.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng primitives.zng foo.zng bar.zng $(ZNGUR)
	$(ZNGUR) g main.zng

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
