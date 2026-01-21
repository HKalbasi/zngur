a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_template_types.a
	${CXX} -std=c++20 -Werror main.cpp -g -L ../../target/release/ -l example_template_types

../../target/release/libexample_template_types.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/template_types/main.zng

.PHONY: ../../target/release/libexample_template_types.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
