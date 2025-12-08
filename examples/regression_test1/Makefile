a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_regression_test1.a
	${CXX} -std=c++20 -Werror -I. main.cpp -g -L ../../target/release/ -l example_regression_test1

../../target/release/libexample_regression_test1.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/regression_test1/main.zng -o ../examples/regression_test1

.PHONY: ../../target/release/libexample_regression_test1.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
