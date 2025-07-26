.PHONY: all clean build_libs build_rlox place_libs 

common_path := build/bin/Debug

all: build_libs build_rlox place_libs

clean:
	cd external/CSR && cmake --build --preset Debug --target clean
	cd external/JASM && cmake --build --preset Debug --target clean
	cd rlox && cargo clean

build_libs:
	cd external/CSR && ./build.sh
	cd external/JASM && ./build.sh

place_libs:
	cp "external/CSR/$(common_path)/csr" "rlox/target/debug/"
	cp "external/CSR/build/lib/libstdjasm/bin/Debug/libstdjasm.so" "rlox/target/debug/"
	cp "external/JASM/$(common_path)/jasm" "rlox/target/debug/"

build_rlox:
	cd rlox && cargo build

test: all
	cd rlox && target/debug/rlox-jasm run test.rlox
