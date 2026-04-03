a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_rayon_split.a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_rayon_split -I .

../../target/release/libexample_rayon_split.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng zngur.h
	cd ../../zngur-cli && cargo run g ../examples/rayon_split/main.zng --crate-name "crate"

zngur.h:
	cd ../../zngur-cli && cargo run h ../examples/rayon_split/zngur.h
	
.PHONY: ../../target/release/libexample_rayon_split.a generated.h clean

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt zngur.h
