a.out: main.cpp generated1.h src/generated1.rs generated2.h src/generated2.rs src/lib.rs ../../target/release/libexample_multiple_zngur_files.a
	${CXX} -std=c++11 -Werror main.cpp -g -L ../../target/release/ -l example_multiple_zngur_files

../../target/release/libexample_multiple_zngur_files.a: FORCE
	cargo build --release

generated1.h ./src/generated1.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/multiple_zngur_files/main.zng \
		--h-file=../examples/multiple_zngur_files/generated1.h \
		--rs-file=../examples/multiple_zngur_files/src/generated1.rs \
		--cpp-namespace=m1

generated2.h ./src/generated2.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/multiple_zngur_files/main.zng \
		--h-file=../examples/multiple_zngur_files/generated2.h \
		--rs-file=../examples/multiple_zngur_files/src/generated2.rs \
		--cpp-namespace=m2

FORCE: ;
