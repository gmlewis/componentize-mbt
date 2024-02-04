use std::env;

use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Config, Engine, Store};

bindgen!("basic" in "wit");

struct MyState {}

impl fantix::examples::stdio::Host for MyState {
    fn println(&mut self, line: String) -> anyhow::Result<()> {
        println!("{line}");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let wasm = env::args().skip(1).next().expect("参数");
    let component = Component::from_file(&engine, wasm)?;
    let mut store = Store::new(&engine, MyState {});
    let mut linker = Linker::new(&engine);
    fantix::examples::stdio::add_to_linker(&mut linker, |state: &mut MyState| state)?;
    let (app, _instance) = Basic::instantiate(&mut store, &component, &linker)?;
    app.call_hello(&mut store, "小熊")?;
    Ok(())
}
