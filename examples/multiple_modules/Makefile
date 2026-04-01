ZNGUR_CLI = cd ../../zngur-cli && /usr/local/google/home/asnaider/.cargo/bin/cargo run

a.out: main.cpp aggregation/impls.cpp aggregation/generated.cpp packet.zng.h receiver.zng.h aggregation.zng.h processor.zng.h zngur.h ../../target/release/libexample_multiple_modules.a
	${CXX} -std=c++17 -Werror -I. -Iaggregation main.cpp aggregation/impls.cpp aggregation/generated.cpp -g -L ../../target/release/ -l example_multiple_modules

../../target/release/libexample_multiple_modules.a:
	/usr/local/google/home/asnaider/.cargo/bin/cargo build --release

packet.zng.h packet/src/packet.zng.rs: packet/packet.zng
	${ZNGUR_CLI} g ../examples/multiple_modules/packet/packet.zng \
		--h-file=../examples/multiple_modules/packet.zng.h \
		--rs-file=../examples/multiple_modules/packet/src/packet.zng.rs

receiver.zng.h receiver/src/receiver.zng.rs: receiver/receiver.zng packet.zng.h
	${ZNGUR_CLI} g ../examples/multiple_modules/receiver/receiver.zng \
		--h-file=../examples/multiple_modules/receiver.zng.h \
		--rs-file=../examples/multiple_modules/receiver/src/receiver.zng.rs

aggregation.zng.h aggregation/src/aggregation.zng.rs: aggregation/aggregation.zng packet.zng.h
	${ZNGUR_CLI} g ../examples/multiple_modules/aggregation/aggregation.zng \
		--h-file=../examples/multiple_modules/aggregation.zng.h \
		--rs-file=../examples/multiple_modules/aggregation/src/aggregation.zng.rs

processor.zng.h processor/src/processor.zng.rs: processor/processor.zng receiver.zng.h aggregation.zng.h
	${ZNGUR_CLI} g ../examples/multiple_modules/processor/processor.zng \
		--h-file=../examples/multiple_modules/processor.zng.h \
		--rs-file=../examples/multiple_modules/processor/src/processor.zng.rs

zngur.h:
	${ZNGUR_CLI} -- make-zng-header --panic-to-exception ../examples/multiple_modules/zngur.h

.PHONY: ../../target/release/libexample_multiple_modules.a clean run

run: a.out
	./a.out

clean:
	rm -f *.zng.h packet/src/*.zng.rs receiver/src/*.zng.rs aggregation/src/*.zng.rs processor/src/*.zng.rs zngur.h a.out
