a.out: main.cpp generated1.h src/generated1.rs generated2.h src/generated2.rs zngur.h src/lib.rs ../../target/release/libexample_multiple_zngur_files.a
	${CXX} -std=c++17 -Werror -I. main.cpp -g -L ../../target/release/ -l example_multiple_zngur_files

../../target/release/libexample_multiple_zngur_files.a:
	cargo build --release

generated1.h ./src/generated1.rs: main1.zng main.zng
	cd ../../zngur-cli && cargo run -- g ../examples/multiple_zngur_files/main1.zng \
		--h-file=../examples/multiple_zngur_files/generated1.h \
		--rs-file=../examples/multiple_zngur_files/src/generated1.rs \
		--cpp-namespace m1 \
		-i

generated2.h ./src/generated2.rs: main2.zng main.zng
	cd ../../zngur-cli && cargo run -- g ../examples/multiple_zngur_files/main2.zng \
		--h-file=../examples/multiple_zngur_files/generated2.h \
		--rs-file=../examples/multiple_zngur_files/src/generated2.rs \
		--cpp-namespace m2 \
		-i

zngur.h:
	cd ../../zngur-cli && cargo run -- make-zng-header --panic-to-exception --cpp-namespace multiple_zngur_files ../examples/multiple_zngur_files/zngur.h


.PHONY: ../../target/release/libexample_multiple_zngur_files.a generated1.h generated2.h clean

clean:
	rm -f generated1.h src/generated1.rs generated2.h src/generated2.rs zngur.h a.out actual_output.txt
