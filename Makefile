# Copyright © 2022 Translucence Research, Inc. All rights reserved.

.PHONY: build
build:
	cargo build --release
	wasm-pack build

.PHONY: clean
clean:
	cargo clean
	cargo clean --release

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

.PHONY: setup
setup:
	curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
