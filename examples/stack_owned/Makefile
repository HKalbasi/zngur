a.out: src/main.rs build.rs main.zng impls.cpp
	cargo build
	cp ../../target/debug/example-stack-owned a.out

.PHONY: clean
clean:
	rm -f a.out actual_output.txt generated.h generated.cpp src/generated.rs
