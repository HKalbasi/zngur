a.out: main.cpp main.zng.h src/main.zng.rs module.zng.h src/module.zng.rs zngur.h ../../target/release/libexample_multiple_modules.a
	${CXX} -std=c++17 -Werror -I. main.cpp -g -L ../../target/release/ -l example_multiple_modules

../../target/release/libexample_multiple_modules.a:
	/usr/local/google/home/asnaider/.cargo/bin/cargo build --release

module.zng.h src/module.zng.rs: module.zng
	cd ../../zngur-cli && /usr/local/google/home/asnaider/.cargo/bin/cargo run g ../examples/multiple_modules/module.zng \
		--h-file=../examples/multiple_modules/module.zng.h \
		--rs-file=../examples/multiple_modules/src/module.zng.rs

main.zng.h src/main.zng.rs: main.zng module.zng.h
	cd ../../zngur-cli && /usr/local/google/home/asnaider/.cargo/bin/cargo run g ../examples/multiple_modules/main.zng \
		--h-file=../examples/multiple_modules/main.zng.h \
		--rs-file=../examples/multiple_modules/src/main.zng.rs

zngur.h:
	cd ../../zngur-cli && /usr/local/google/home/asnaider/.cargo/bin/cargo run -- make-zng-header --panic-to-exception ../examples/multiple_modules/zngur.h

.PHONY: ../../target/release/libexample_multiple_modules.a clean

clean:
	rm -f *.zng.h src/*.zng.rs zngur.h a.out
