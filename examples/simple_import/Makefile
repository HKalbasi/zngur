.PHONY: ../../target/release/libexample_simple_import.a

a.out: main.cpp foo.cpp bar.cpp generated.h zngur.h src/generated.rs src/lib.rs ../../target/release/libexample_simple_import.a
	${CXX} -std=c++17 -Werror -I. main.cpp foo.cpp bar.cpp -g -L ../../target/release/ -l example_simple_import

../../target/release/libexample_simple_import.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng primitives.zng foo.zng bar.zng
	cd ../../zngur-cli && cargo run g ../examples/simple_import/main.zng

zngur.h:
	cd ../../zngur-cli && cargo run -- make-zng-header --panic-to-exception ../examples/simple_import/zngur.h

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
