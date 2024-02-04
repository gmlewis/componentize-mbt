use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use wast::core::{
    Func, FuncKind, Import, InlineExport, Instruction, Memory, Module, ModuleField, ModuleKind,
};
use wast::parser::ParseBuffer;
use wast::token::Index;
use wast::Wat;
use wit_bindgen_core::wit_parser::{
    Function, FunctionKind, Resolve, Results, WorldId, WorldItem, WorldKey,
};
use wit_bindgen_core::{Files, WorldGenerator};
use wit_component::{embed_component_metadata, ComponentEncoder, StringEncoding};

#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Opts {
    #[clap(long)]
    wat: PathBuf,
}

impl Opts {
    pub fn run(
        &self,
        resolve: Resolve,
        world: WorldId,
        out_dir: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        let wat = fs::read_to_string(&self.wat)?;
        let wasm = componentize(&wat, resolve, world)?;
        let target = self.wat.with_extension("wasm");
        let target = match out_dir {
            Some(out_dir) => out_dir.join(target.file_name().unwrap()),
            None => target,
        };
        fs::write(&target, wasm)?;
        println!("Write to {target:?}");
        Ok(())
    }
}

const MBT_INIT: &str = "mbt-init";

pub fn componentize(wat: &str, mut resolve: Resolve, world: WorldId) -> anyhow::Result<Vec<u8>> {
    // 如果不存在，添加 export mbt-init: func();
    if resolve.worlds[world]
        .exports
        .values()
        .find(|item| {
            matches!(
                item,
                WorldItem::Function(Function {
                    name,
                    ..
                }) if name == MBT_INIT
            )
        })
        .is_none()
    {
        resolve.worlds[world].exports.insert(
            WorldKey::Name(MBT_INIT.into()),
            WorldItem::Function(Function {
                name: MBT_INIT.into(),
                kind: FunctionKind::Freestanding,
                params: vec![],
                results: Results::Named(Default::default()),
                docs: Default::default(),
            }),
        );
    }

    // 再跑一遍 bindgen 取回 exported_symbols，等到 MoonBit 支持自定义 FFI export 名称后，删除此 HACK
    let mut gen = wit_bindgen_mbt::MoonBit::default();
    gen.generate(&resolve, world, &mut Files::default())?;
    let exported_symbols = gen.exported_symbols;

    // 一些用于 lift/lower 的 ABI 函数，等到 MoonBit 能直接支持了，就可以删除了
    let mut impls = HashMap::from([
        (
            "rael.memory_copy",
            ParseBuffer::new(
                "func $rael.memory_copy \
                        (param $dest i32) (param $src i32) (param $len i32) \
                        (memory.copy \
                            (local.get $dest) (local.get $src) (local.get $len)\
                        )",
            )?,
        ),
        (
            "rael.load_i32",
            ParseBuffer::new(
                "func $rael.load_i32 \
                        (param $ptr i32) (result i32) \
                        (i32.load (local.get $ptr))",
            )?,
        ),
        (
            "rael.load_i64",
            ParseBuffer::new(
                "func $rael.load_i64 \
                        (param $ptr i32) (result i64) \
                        (i64.load (local.get $ptr))",
            )?,
        ),
        (
            "rael.bytes_data",
            ParseBuffer::new(
                "func $rael.bytes_data \
                        (param $str i32) (result i32) \
                        (i32.add (local.get $str) (i32.const 4))",
            )?,
        ),
        (
            "moonbit.string_data",
            ParseBuffer::new(
                "func $moonbit.string_data \
                        (param $str i32) (result i32) \
                        (i32.add (local.get $str) (i32.const 4))",
            )?,
        ),
        ("printc", ParseBuffer::new("func $printc (param $ptr i32)")?),
    ]);
    let mut builtins = HashMap::new();
    let mut realloc = Some(ParseBuffer::new(
        "func (export \"cabi_realloc\") \
            (param i32) (param i32) (param i32) (param $len i32) (result i32) \
            (call $rael.malloc (local.get $len))",
    )?);

    let buf = ParseBuffer::new(wat)?;
    let mut ast = wast::parser::parse(&buf)?;
    match &mut ast {
        Wat::Module(Module {
            kind: ModuleKind::Text(ref mut fields),
            ..
        }) => {
            fields.retain(|field| {
                !matches!(
                    field,
                    ModuleField::Import(Import {
                        module: "spectest",
                        ..
                    })
                )
            });
            for field in fields.iter_mut() {
                match field {
                    ModuleField::Memory(Memory {
                        exports: InlineExport { names },
                        ..
                    }) if names == &vec!["moonbit.memory"] => {
                        names[0] = "memory";
                    }
                    ModuleField::Func(Func {
                        kind: FuncKind::Inline { expression, .. },
                        exports,
                        ..
                    }) => {
                        if exports.names.len() == 1 {
                            if let Some((_, key)) = exports.names[0].split_once("::") {
                                if let Some(symbol) = exported_symbols.get(key) {
                                    exports.names[0] = symbol
                                }
                            }
                        }
                        for instr in expression.instrs.iter() {
                            match instr {
                                Instruction::Call(Index::Id(id)) => {
                                    let name = id.name();
                                    if !builtins.contains_key(name) {
                                        if let Some((name, imp)) = impls.remove_entry(name) {
                                            builtins.insert(name, imp);
                                        }
                                    }
                                    if name == "rael.malloc" {
                                        if let Some(realloc) = realloc.take() {
                                            builtins.insert(name, realloc);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    ModuleField::Export(e) if e.name == "_start" => {
                        e.name = MBT_INIT;
                    }
                    _ => {}
                }
            }

            for buf in builtins.values() {
                let field: ModuleField = wast::parser::parse(buf)?;
                fields.push(field);
            }
        }
        _ => {}
    }

    let mut buf = ast.encode()?;
    embed_component_metadata(&mut buf, &resolve, world, StringEncoding::UTF8)?;
    Ok(ComponentEncoder::default().module(&buf)?.encode()?)
}
