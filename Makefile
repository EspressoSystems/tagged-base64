# Copyright © 2022 Translucence Research, Inc. All rights reserved.

.PHONY: build
build:
	cargo build --release
	wasm-pack build

.PHONY: clean
clean:
	cargo clean
	cargo clean --release
	rm -rf ./target grcov-*.profraw

.PHONY: check
check:
	cargo check


.PHONY: test
test:
	cargo test --release
	wasm-pack test --headless --firefox --chrome

.PHONY: doc
doc:
	cargo doc --no-deps

.PHONY: coverage
coverage: export RUSTUP_TOOLCHAIN=nightly
coverage: export LLVM_PROFILE_FILE=grcov-%p-%m.profraw
coverage: export RUSTFLAGS=-Zinstrument-coverage
coverage:
	rm -rf grcov-*.profraw default.profraw
	cargo test
	grcov .                                \
	    --binary-path ./target/debug/      \
	    -s .                               \
	    -t html                            \
	    --branch                           \
	    --ignore-not-existing              \
	    -o ./coverage/ &&                  \
	echo "See file://${PWD}/coverage/index.html for coverage."

.PHONY: setup
setup:
	curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
