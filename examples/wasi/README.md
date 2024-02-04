# WASI 示例

先安装 `wit-deps` 的命令行工具：

```
cargo install wit-deps-cli
```

然后执行 `make` 运行测试：

```
$ make
wit-deps lock
cargo run --manifest-path ../../Cargo.toml -- bindgen wit --out-dir main
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/componentize-mbt-cli bindgen wit --out-dir main`

Generating "main/wasi_demo.mbt"
moon build --output-wat
moonc build-package examples/wasi/main/wasi_demo.mbt examples/wasi/main/main.mbt -o examples/wasi/target/wasm/release/build/main/main.core -pkg fantix/component-wasi-example/main -pkg-sources fantix/component-wasi-example/main:examples/wasi/main
examples/wasi/main/main.mbt:3:13-3:17 Warning 002: Unused variable 'self'
examples/wasi/main/wasi_demo.mbt:35:25-35:29 Warning 002: Unused variable 'self'
examples/wasi/main/wasi_demo.mbt:52:23-52:27 Warning 002: Unused variable 'self'
moon: ran 2 tasks, now up to date

cargo run --manifest-path ../../Cargo.toml -- componentize wit --wat target/wasm/release/build/main/main.wat
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/componentize-mbt-cli componentize wit --wat target/wasm/release/build/main/main.wat`
Write to "target/wasm/release/build/main/main.wasm"

cargo run -- target/wasm/release/build/main/main.wasm
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/wasi target/wasm/release/build/main/main.wasm`
Nonsense: 4258729435058112404
Nonsense: 3292553995670347341
Nonsense: 11390975368387254577
Nonsense: 7512815099977197841
Nonsense: 8623039804862391839
Nonsense: 908748041087640565
Nonsense: 8717367353637816312
Nonsense: 8606579203021978716
Nonsense: 153497577793621678
Nonsense: 5445473361388511267
```
