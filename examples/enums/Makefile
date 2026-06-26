a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_enums.a
	${CXX} -std=c++17 -Werror main.cpp  -g -L ../../target/release/ -l example_enums

../../target/release/libexample_enums.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g -i ../examples/enums/main.zng --crate-name "crate"

.PHONY: ../../target/release/libexample_enums.a generated.h clean

clean:
	rm -f generated.h src/generated.rs a.out actual_output.txt
