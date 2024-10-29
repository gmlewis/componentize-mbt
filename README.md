# componentize-mbt - ARCHIVED

This is a translation from Chinese to English of this repo: https://gitee.com/fantix/componentize-mbt

A temporary command-line tool for building components before [MoonBit](https://www.moonbitlang.com/) officially supports the [component model](https://github.com/WebAssembly/component-model).

## Usage

1. Install: `cargo install componentize-mbt-cli`;
2. Create a `wit` folder in the root directory of your MoonBit project;
3. Create `.wit` files as needed in the `wit` folder;
4. If necessary, create a `wit/deps.toml` file and install `cargo install wit-deps-cli` for dependency management, such as importing WASI interfaces;
5. Run `componentize-mbt bindgen --out-dir ...` to generate MoonBit binding code corresponding to the WIT;
6. Use the newly generated code to complete project functionality; if interfaces are exported in WIT, implement the corresponding traits and call `init_guest()` to set up the implementation instance;
7. Run `componentize-mbt` to build the component.

Step 7 is equivalent to the following two steps:

1. Run `moon build --output-wat` to compile to WAT (using WAT instead of WASM here takes advantage of a hidden flaw in MoonBit: generating WAT doesn't check ABI imports, making it easier for us to link component-related WASM code in the next step);
2. Run `componentize-mbt componentize wit --wat ...` to wrap the WAT into a component WASM.

## `bind-gen`

Reads [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md) files and generates MoonBit binding code.

### Import Generation Example

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

### Export Generation Example

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

// User-side code

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

Takes WIT and a `.wat` file as input, and synthesizes a `.wasm` file conforming to the [component model specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md). Implementation process:

1. Re-execute bindgen in memory to find and replace export symbols (can be removed once MoonBit supports custom FFI export names);
2. Remove spectest imports (will be changed to WIT imports later);
3. Rename `moonbit.memory` to `memory`;
4. Add WASM functions required for component lift/lower;
5. Fix the issue where MoonBit compiles pub fn with no return value to return i32 (?);
6. Change `export _start` to `start`;
7. Add component encapsulation;
8. Export the `.wasm` file.
