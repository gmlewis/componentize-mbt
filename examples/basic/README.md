# Basic Example

Run `make` to perform the test:

```
$ make

cargo run --manifest-path ../../Cargo.toml -- bindgen wit --out-dir main
   Compiling wit-bindgen-mbt v0.1.0 (crates/bindgen)
   Compiling componentize-mbt v0.1.0 (crates/componentize)
   Compiling componentize-mbt-cli v0.1.0 (.)
    Finished dev [unoptimized + debuginfo] target(s) in 1.41s
     Running `target/debug/componentize-mbt-cli bindgen wit --out-dir main`
Generating "main/basic.mbt"

moon build --output-wat
moonc build-package examples/basic/main/basic.mbt examples/basic/main/main.mbt -o examples/basic/target/wasm/release/build/main/main.core -pkg fantix/component-basic-example/main -pkg-sources fantix/component-basic-example/main:examples/basic/main
examples/basic/main/main.mbt:3:17-3:21 Warning 002: Unused variable 'self'
examples/basic/main/basic.mbt:26:16-26:20 Warning 002: Unused variable 'self'
moon: ran 2 tasks, now up to date

cargo run --manifest-path ../../Cargo.toml -- componentize wit --wat target/wasm/release/build/main/main.wat
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/componentize-mbt-cli componentize wit --wat target/wasm/release/build/main/main.wat`
Write to "target/wasm/release/build/main/main.wasm"

cargo run -- target/wasm/release/build/main/main.wasm
   Compiling basic v0.1.0 (examples/basic)
    Finished dev [unoptimized + debuginfo] target(s) in 2.57s
     Running `target/debug/basic target/wasm/release/build/main/main.wasm`
Hello, little bear!
```
