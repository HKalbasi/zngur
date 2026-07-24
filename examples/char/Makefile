EXAMPLES_CARGO_PROFILE ?= debug

a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_char.a
	${CXX} -std=c++11 -Werror -I. main.cpp -g -L ../../target/$(EXAMPLES_CARGO_PROFILE)/ -l example_char

../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_char.a:
	cargo build --profile $(EXAMPLES_CARGO_PROFILE:debug=dev)

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g -i ../examples/char/main.zng --crate-name "crate"

.PHONY: ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_char.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
