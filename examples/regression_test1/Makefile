a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_regression_test1.a
	${CXX} -Werror main.cpp -g -L ../../target/release/ -l example_regression_test1

../../target/release/libexample_regression_test1.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/regression_test1/main.zng

FORCE: ;
