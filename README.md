# componentize-mbt

在 MoonBit 正式支持
[component model](https://github.com/WebAssembly/component-model)
之前，临时用于构建 component 的命令行工具。

## 用法

#. 在 MoonBit 项目根目录下，创建 `wit` 文件夹；
#. 在 `wit` 文件夹下，按需创建 `.wit` 文件；
#. 如有需要，可创建 `wit/deps.toml` 文件，安装 `cargo install wit-deps-cli`
   进行依赖管理，比如引入 WASI 的接口；
#. 执行 `cargo run -- bindgen wit -w ...` 生成 WIT 对应的 MoonBit 绑定代码；
#. 使用新生成的代码，完成项目功能；如在 WIT 中 export 了接口，则须实现对应的 trait，并调用
   `init_guest()` 安置实现实例；
#. 执行 `moon build --output-wat` 编译出 WAT（这里之所以不用 wasm，是利用了 MoonBit
   的一个隐藏缺陷，生成 WAT 时并不会检查 ABI import，便于我们下一步链接 component 相关的
   wasm 代码）；
#. 执行 `cargo run -- componentize wit --wat ...`，将 WAT 包装成 component WASM。

## `bind-gen`

读取 [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
文件，生成 MoonBit 的 binding 代码。

### Import 生成示例

```
pub(readonly) struct Wasi {
  clocks : WasiClocks
}

pub(readonly) struct WasiClocks {
  monotonic_clock : WasiClocksMonotonicClock
}

pub(readonly) type WasiClocksMonotonicClock Unit

pub fn now(self : WasiClocksMonotonicClock) -> Int64 {
  // Impl
  0L
}

pub let wasi : Wasi = Wasi::{
  clocks: WasiClocks::{ monotonic_clock: WasiClocksMonotonicClock(()) },
}

test "use import" {
  if wasi.clocks.monotonic_clock.now() != 0L {
    abort("failed")
  }
}
```

### Export 生成示例

```
pub trait ExportsWasiClocksMonotonicClock  {
  now(Self) -> Int64
}

struct GuestImpl {
  mut wasi_clocks_monotonic_clock: Option[ExportsWasiClocksMonotonicClock]
} derive(Default)

let guest_impl: GuestImpl = GuestImpl::default()

pub fn init_guest[T: ExportsWasiClocksMonotonicClock](guest: T) {
  guest_impl.wasi_clocks_monotonic_clock = Some(guest as ExportsWasiClocksMonotonicClock)
}

// export_name = "now"
pub fn __export_now() -> Int64 {
  guest_impl.wasi_clocks_monotonic_clock.unwrap().now()
}

// 用户端代码

struct App {}

fn ExportsWasiClocksMonotonicClock::now(self: App) -> Int64 {
  // Impl
  0L
}

let app: App = App::{}

fn init {
  init_guest(app)
}

test "extern call" {
  if ffi_now() != 0L {
    abort("failed")
  }
}
```

## `componentize`

输入 WIT 和一个 `.wat` 文件，合成出符合
[component model 规范](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)的
`.wasm` 文件。实现流程：

#. 在内存中重新执行 bindgen，找到 export symbol；
#. 将 spectest 的 import 删掉（之后会改成 WIT import）；
#. 将 `moonbit.memory` 重命名为 `memory`；
#. 增加 component lift/lower 所需的 WASM 函数；
#. 将 `export _start` 修改为 `start`；
#. 增加 component 封装；
#. 导出 `.wasm` 文件。
