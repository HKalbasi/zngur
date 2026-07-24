EXAMPLES_CARGO_PROFILE ?= debug

a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_simple.a
	${CXX} -std=c++17 -Werror main.cpp generated.cpp -g -L ../../target/$(EXAMPLES_CARGO_PROFILE)/ -l example_simple

../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_simple.a:
	cargo build --profile $(EXAMPLES_CARGO_PROFILE:debug=dev)

generated.h generated.cpp ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g -i ../examples/simple/main.zng --crate-name "crate"

.PHONY: ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_simple.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
