EXAMPLES_CARGO_PROFILE ?= debug

.PHONY: ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_simple_import.a

a.out: main.cpp foo.cpp bar.cpp generated.h src/generated.rs src/lib.rs ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_simple_import.a
	${CXX} -std=c++11 main.cpp foo.cpp bar.cpp -g -L ../../target/$(EXAMPLES_CARGO_PROFILE)/ -l example_simple_import

../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_simple_import.a:
	cargo build --profile $(EXAMPLES_CARGO_PROFILE:debug=dev)

generated.h ./src/generated.rs: main.zng primitives.zng foo.zng bar.zng
	cd ../../zngur-cli && cargo run g -i ../examples/simple_import/main.zng --crate-name "crate"

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
