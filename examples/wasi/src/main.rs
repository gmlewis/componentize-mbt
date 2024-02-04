use std::env;

use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{bindings, WasiCtx, WasiCtxBuilder, WasiView};

bindgen!("wasi-demo" in "wit");

struct MyState {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for MyState {
    fn table(&self) -> &ResourceTable {
        &self.table
    }

    fn table_mut(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl Default for MyState {
    fn default() -> Self {
        Self {
            wasi: WasiCtxBuilder::new().build(),
            table: Default::default(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let wasm = env::args().skip(1).next().expect("参数");
    let component = Component::from_file(&engine, wasm)?;
    let mut store = Store::new(&engine, MyState::default());
    let mut linker = Linker::new(&engine);
    bindings::random::random::add_to_linker(&mut linker, |s| s)?;
    let (app, _instance) = WasiDemo::instantiate(&mut store, &component, &linker)?;
    for _ in 0..10 {
        let ns = app.call_nonsense(&mut store)?;
        println!("Nonsense: {ns}");
    }
    Ok(())
}
