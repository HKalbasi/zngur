a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_impl_trait.a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_impl_trait

../../target/release/libexample_impl_trait.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/impl-trait/main.zng

.PHONY: ../../target/release/libexample_impl_trait.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
