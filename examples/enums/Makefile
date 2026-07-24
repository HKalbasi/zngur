EXAMPLES_CARGO_PROFILE ?= debug

a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_enums.a
	${CXX} -std=c++17 -Werror main.cpp  -g -L ../../target/$(EXAMPLES_CARGO_PROFILE)/ -l example_enums

../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_enums.a:
	cargo build --profile $(EXAMPLES_CARGO_PROFILE:debug=dev)

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g -i ../examples/enums/main.zng --crate-name "crate"

.PHONY: ../../target/$(EXAMPLES_CARGO_PROFILE)/libexample_enums.a generated.h clean

clean:
	rm -f generated.h src/generated.rs a.out actual_output.txt
