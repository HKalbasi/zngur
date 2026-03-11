a.out: main.cpp generated.h zngur.h src/generated.rs src/lib.rs ../../target/release/libexample_rayon.a
	${CXX} -std=c++17 -Werror -I. main.cpp -g -L ../../target/release/ -l example_rayon

../../target/release/libexample_rayon.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/rayon/main.zng

zngur.h:
	cd ../../zngur-cli && cargo run -- make-zng-header --panic-to-exception ../examples/rayon/zngur.h

.PHONY: ../../target/release/libexample_rayon.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs zngur.h a.out actual_output.txt
