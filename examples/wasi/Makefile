test:
	wit-deps lock
	cargo run --manifest-path ../../Cargo.toml -- bindgen wit --out-dir main
	moon build --output-wat
	cargo run --manifest-path ../../Cargo.toml -- componentize wit --wat target/wasm/release/build/main/main.wat
	cargo run -- target/wasm/release/build/main/main.wasm
