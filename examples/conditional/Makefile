both:: a.out b.out

a.out: main.cpp include/float/generated.h src/lib.rs ../../target/release/libexample_conditional.a
	${CXX} -std=c++20 -Werror main.cpp -g -I include/float -o a.out -L../../target/release -l example_conditional

b.out: main.cpp include/int/generated.h src/lib.rs ../../target/debug/libexample_conditional.a
	${CXX} -std=c++20 -Werror main.cpp -g -I include/int -o b.out -L../../target/debug -l example_conditional

../../target/release/libexample_conditional.a: ./src/generated_float.rs
	cargo build --release -F "float-values"

../../target/debug/libexample_conditional.a: ./src/generated_int.rs
	cargo build

include/float/generated.h ./src/generated_float.rs: main.zng
	mkdir -p include/float
	cd ../../zngur-cli && cargo run g ../examples/conditional/main.zng --h-file ../examples/conditional/include/float/generated.h --rs-file ../examples/conditional/src/generated_float.rs -F "float-values" --load-cfg-from-rustc --use-cargo-rustc --profile=release --package example-conditional

include/int/generated.h ./src/generated_int.rs: main.zng
	mkdir -p include/int
	cd ../../zngur-cli && cargo run g ../examples/conditional/main.zng --h-file ../examples/conditional/include/int/generated.h --rs-file ../examples/conditional/src/generated_int.rs --load-cfg-from-rustc

.PHONY: ../../target/release/libexample_conditional.a ../../target/debug/libexample_conditional.a include/float/generated.h include/int/generated.h clean

clean:
	rm -f include/float/generated.h include/int/generated.h generated.cpp src/generated_float.rs src/generated_int.rs a.out b.out actual_output.txt
