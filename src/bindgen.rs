use clap::Args;
use wit_bindgen_core::{Files, WorldGenerator};
use wit_parser::{Function, InterfaceId, Resolve, TypeId, WorldId, WorldKey};

#[derive(Default)]
struct MoonBit {
    opts: Opts,
}

#[derive(Default, Debug, Clone, Args)]
pub struct Opts {}

impl Opts {
    pub fn build(&self) -> Box<dyn WorldGenerator> {
        let mut r = MoonBit::default();
        r.opts = self.clone();
        Box::new(r)
    }
}

impl WorldGenerator for MoonBit {
    fn import_interface(
        &mut self,
        resolve: &Resolve,
        name: &WorldKey,
        iface: InterfaceId,
        files: &mut Files,
    ) {
        todo!()
    }

    fn export_interface(
        &mut self,
        resolve: &Resolve,
        name: &WorldKey,
        iface: InterfaceId,
        files: &mut Files,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn import_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        files: &mut Files,
    ) {
        todo!()
    }

    fn export_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        files: &mut Files,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn import_types(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        types: &[(&str, TypeId)],
        files: &mut Files,
    ) {
        todo!()
    }

    fn finish(&mut self, resolve: &Resolve, world: WorldId, files: &mut Files) {
        todo!()
    }
}
