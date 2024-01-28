# componentize-mbt

在 MoonBit 正式支持
[component model](https://github.com/WebAssembly/component-model)
之前，临时用于构建 component 的命令行工具。

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
  handle(Self) -> Int64
}

fn wrap_ExportsWasiClocksMonotonicClock_handle[T : ExportsWasiClocksMonotonicClock](impl : T) -> Int64 {
  impl.handle()
}

fn ffi_handle() -> Int64 {
  wrap_ExportsWasiClocksMonotonicClock_handle(app)
}

// 用户端代码

struct App {}

fn ExportsWasiClocksMonotonicClock::handle(self: App) -> Int64 {
  // Impl
  0L
}

let app: App = App::{}

test "extern call" {
  if ffi_handle() != 0L {
    abort("failed")
  }
}
```

## `build`

构建 MoonBit 项目，并合成出符合
[component model 规范](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)的
`.wasm` 文件。
