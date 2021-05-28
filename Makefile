# Copyright © 2021 Translucence Research, Inc. All rights reserved.

.PHONY: build
build:
	cargo build --release
	wasm-pack build

.PHONY: check
check:
	cargo check


.PHONY: test
test:
	wasm-pack test --headless --firefox --chrome
