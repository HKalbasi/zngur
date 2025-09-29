a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_regression_test1.a
	${CXX} -std=c++20 -Werror main.cpp -g -L ../../target/release/ -l example_regression_test1

../../target/release/libexample_regression_test1.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng FORCE
	cd ../../zngur-cli && cargo run g ../examples/regression_test1/main.zng

FORCE: ;

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
