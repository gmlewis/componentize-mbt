use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, mem};

use wast::core::{
    Func, FuncKind, Import, InlineExport, Instruction, Memory, Module, ModuleField, ModuleKind,
};
use wast::parser::ParseBuffer;
use wast::token::Index;
use wast::Wat;
use wit_bindgen_core::wit_parser::{Resolve, WorldId};
use wit_bindgen_core::{Files, WorldGenerator};
use wit_component::{embed_component_metadata, ComponentEncoder, StringEncoding};

#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Opts {
    #[cfg_attr(feature = "clap", arg(long))]
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

pub fn componentize(wat: &str, resolve: Resolve, world: WorldId) -> anyhow::Result<Vec<u8>> {
    // Run `bindgen` again to retrieve exported_symbols. This HACK can be removed once MoonBit supports custom FFI export names
    let mut gen = wit_bindgen_mbt::MoonBit::default();
    gen.generate(&resolve, world, &mut Files::default())?;
    let exported_symbols = gen.exported_symbols;

    // Some ABI functions for lift/lower. These can be removed once MoonBit directly supports them
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
    let mut start = None;

    if let Wat::Module(Module {
        kind: ModuleKind::Text(ref mut fields),
        ..
    }) = &mut ast
    {
        fields.retain_mut(|field| match field {
            ModuleField::Import(Import {
                module: "spectest", ..
            }) => false,
            ModuleField::Memory(Memory {
                exports: InlineExport { names },
                ..
            }) if names == &vec!["moonbit.memory"] => {
                names[0] = "memory";
                true
            }
            ModuleField::Func(Func {
                kind: FuncKind::Inline { expression, .. },
                exports,
                ty,
                ..
            }) => {
                if exports.names.len() == 1 {
                    if let Some((_, key)) = exports.names[0].split_once("::") {
                        if let Some((symbol, has_rv)) = exported_symbols.get(key) {
                            exports.names[0] = symbol;

                            // This seems to be a bug in MoonBit - pub fn has no return value, but the WASM func returns i32
                            // This was fixed in the 2024-06-25 version of the MonnBit compiler.
                            if !has_rv {
                                if let Some(ty) = &mut ty.inline {
                                    if ty.results.len() == 1 {
                                        ty.results = Box::new([]);
                                        let mut instrs = Default::default();
                                        mem::swap(&mut expression.instrs, &mut instrs);
                                        let mut instrs = instrs.into_vec();
                                        instrs.push(Instruction::Drop);
                                        expression.instrs = instrs.into();
                                    }
                                }
                            }
                        }
                    }
                }
                for instr in expression.instrs.iter() {
                    if let Instruction::Call(Index::Id(id)) = instr {
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
                }
                true
            }
            ModuleField::Export(e) if e.name == "_start" => {
                start = Some(e.item);
                false
            }
            _ => true,
        });

        for buf in builtins.values() {
            let field: ModuleField = wast::parser::parse(buf)?;
            fields.push(field);
        }
        fields.push(ModuleField::Start(start.expect("_start")));
    }

    let mut buf = ast.encode()?;
    embed_component_metadata(&mut buf, &resolve, world, StringEncoding::UTF8)?;
    ComponentEncoder::default().module(&buf)?.encode()
}
