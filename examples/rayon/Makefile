a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_rayon.a
	${CXX} -Werror main.cpp -g -L ../../target/release/ -l example_rayon

../../target/release/libexample_rayon.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/rayon/main.zng

FORCE: ;
