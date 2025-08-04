.PHONY: all clean build_libs build_rlox place_libs 

common_path := build/bin/Release
SHELL := /bin/bash

all: build_libs build_rlox place_libs

clean:
	cd external/CSR && cmake --build --preset Debug --target clean
	cd external/JASM && cmake --build --preset Debug --target clean
	cd rlox && cargo clean

build_libs:
	cd external/CSR && ./build.sh -R
	cd external/JASM && ./build.sh -R

place_libs:
	cp "external/CSR/$(common_path)/csr" "rlox/target/release/"
	cp "external/JASM/$(common_path)/jasm" "rlox/target/release/"

build_rlox:
	cd rlox && cargo build --release

test: all
	cd rlox && time target/release/rlox-jasm run test.rlox

interpret: build_rlox
	cd rlox && target/release/rlox-jasm run interpret test2.rlox
