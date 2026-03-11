a.out: main.cpp generated.h zngur.h src/generated.rs src/lib.rs ../../target/release/libexample_impl_trait.a
	${CXX} -std=c++20 -Werror -I. main.cpp -g -L ../../target/release/ -l example_impl_trait

../../target/release/libexample_impl_trait.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/impl_trait/main.zng

zngur.h:
	cd ../../zngur-cli && cargo run -- make-zng-header --panic-to-exception ../examples/impl_trait/zngur.h

.PHONY: ../../target/release/libexample_impl_trait.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
