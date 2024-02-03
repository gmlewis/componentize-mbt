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

## `build`

构建 MoonBit 项目，并合成出符合
[component model 规范](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)的
`.wasm` 文件。
