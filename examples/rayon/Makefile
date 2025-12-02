ZNGUR ?= ../../target/debug/zngur

a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_rayon.a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_rayon

../../target/release/libexample_rayon.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng $(ZNGUR)
	$(ZNGUR) g main.zng

.PHONY: ../../target/release/libexample_rayon.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
