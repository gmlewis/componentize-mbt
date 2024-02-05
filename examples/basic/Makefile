test:
	cargo run --manifest-path ../../Cargo.toml -- bindgen --out-dir main
	cargo run --manifest-path ../../Cargo.toml
	cargo run -- target/wasm/release/build/main/main.wasm
