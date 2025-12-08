a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_rustyline.a
	${CXX} -std=c++11 -Werror -I. main.cpp -g -L ../../target/release/ -l example_rustyline

../../target/release/libexample_rustyline.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/rustyline/main.zng -o ../examples/rustyline

.PHONY: ../../target/release/libexample_rustyline.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
