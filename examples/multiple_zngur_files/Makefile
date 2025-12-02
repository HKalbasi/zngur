ZNGUR ?= ../../target/debug/zngur

a.out: main.cpp generated1.h src/generated1.rs generated2.h src/generated2.rs src/lib.rs ../../target/release/libexample_multiple_zngur_files.a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_multiple_zngur_files

../../target/release/libexample_multiple_zngur_files.a:
	cargo build --release

generated1.h ./src/generated1.rs: main.zng $(ZNGUR)
	$(ZNGUR) g main.zng \
		--h-file=generated1.h \
		--rs-file=src/generated1.rs \
		--cpp-namespace=m1

generated2.h ./src/generated2.rs: main.zng $(ZNGUR)
	$(ZNGUR) g main.zng \
		--h-file=generated2.h \
		--rs-file=src/generated2.rs \
		--cpp-namespace=m2

.PHONY: ../../target/release/libexample_multiple_zngur_files.a generated1.h generated2.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
