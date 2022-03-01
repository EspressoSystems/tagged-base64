#/usr/bin/env bash
set -euo pipefail
cargo build --release --target wasm32-unknown-unknown
mkdir -p js-test/tagged-base64
wasm-bindgen --omit-imports --target web target/wasm32-unknown-unknown/release/tagged-base64.wasm --out-dir js-test/tagged-base64
pushd js-test
exec python -m http.server

