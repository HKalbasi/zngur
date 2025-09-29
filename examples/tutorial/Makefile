a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_tutorial.a
	${CXX} -Werror main.cpp -g -L ../../target/release/ -l example_tutorial

../../target/release/libexample_tutorial.a: FORCE
	cargo build --release

generated.h ./src/generated.rs: main.zng FORCE
	cd ../../zngur-cli && cargo run g ../examples/tutorial/main.zng

FORCE: ;

clean:
	rm -f generated.h generated.cpp src/generated.rs a.out actual_output.txt
