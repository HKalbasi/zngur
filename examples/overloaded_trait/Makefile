NAME = overloaded_trait

a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_$(NAME).a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_$(NAME)

../../target/release/libexample_$(NAME).a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g -i ../examples/$(NAME)/main.zng --crate-name "crate"

.PHONY: ../../target/release/libexample_$(NAME).a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
