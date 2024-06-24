use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use wit_bindgen_core::Files;
use wit_parser::{Resolve, UnresolvedPackage, WorldId};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    world: Option<String>,

    #[command(subcommand)]
    opts: Option<Opt>,
}

#[derive(Subcommand)]
enum Opt {
    Bindgen {
        #[clap(flatten)]
        opts: wit_bindgen_mbt::Opts,

        #[clap(flatten)]
        args: Common,
    },
    Componentize {
        #[clap(flatten)]
        opts: componentize_mbt::Opts,

        #[clap(flatten)]
        args: Common,
    },
}

#[derive(Debug, Parser)]
struct Common {
    #[clap(long = "out-dir")]
    out_dir: Option<PathBuf>,

    #[clap(value_name = "DOCUMENT", index = 1, default_value = "wit")]
    wit: PathBuf,
}

impl Common {
    fn parse_wit(&self, world: Option<&str>) -> Result<(Resolve, WorldId)> {
        let mut resolve = Resolve::default();
        let pkg = if self.wit.is_dir() {
            resolve.push_dir(&self.wit)?.0
        } else {
            resolve.push(UnresolvedPackage::parse_file(&self.wit)?)?
        };
        let world = resolve.select_world(pkg, world)?;
        Ok((resolve, world))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let world = cli.world.as_deref();
    match cli.opts {
        Some(Opt::Bindgen { opts, args }) => {
            let mut files = Files::default();
            let (resolve, world) = args.parse_wit(world)?;
            opts.build().generate(&resolve, world, &mut files)?;
            for (name, contents) in files.iter() {
                let dst = match &args.out_dir {
                    Some(ref path) => path.join(name),
                    None => name.into(),
                };
                println!("Generating {:?}", dst);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create {:?}", parent))?;
                }
                fs::write(&dst, contents).with_context(|| format!("failed to write {:?}", dst))?;
            }
        }
        Some(Opt::Componentize { opts, args }) => {
            let (resolve, world) = args.parse_wit(world)?;
            opts.run(resolve, world, args.out_dir)?;
        }
        None => build(world)?,
    }

    Ok(())
}

fn build(world: Option<&str>) -> Result<()> {
    if !PathBuf::from("moon.mod.json").exists() {
        anyhow::bail!("You must execute componentize-mbt build in the project root directory!");
    }
    let mut iter = fs::read_dir(".")?
        .map(|r| -> Result<_> {
            let r = r?;
            let p = r.path();
            let j = p.join("moon.pkg.json");
            if !j.exists() {
                return Ok(None);
            }
            let json = fs::read_to_string(&j)?;
            let json: serde_json::Value = serde_json::from_str(&json)?;
            let json = json
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("{j:?} Format error!"))?;
            if let Some(is_main) = json.get("is_main") {
                let is_main = is_main
                    .as_bool()
                    .ok_or_else(|| anyhow::anyhow!("{j:?} Format error!"))?;
                Ok(is_main.then(|| r.file_name()))
            } else {
                Ok(None)
            }
        })
        .map(|r| r.transpose())
        .flatten();
    let pkg_name = iter.next().transpose()?.ok_or_else(|| {
        anyhow::anyhow!("At least one MoonBit package must have is_main set to true.")
    })?;
    if iter.next().transpose()?.is_some() {
        anyhow::bail!("Currently, only one MoonBit package can have is_main set to true.");
    }

    let mut cmd = Command::new("moon");
    let cmd = cmd.arg("build").arg("--output-wat");
    println!("Execute: {cmd:?}");
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("moon build failed");
    }

    let wat_file = PathBuf::from("target/wasm/release/build/")
        .join(&pkg_name)
        .join(&pkg_name)
        .with_extension("wat");
    if !wat_file.exists() {
        anyhow::bail!("{wat_file:?} does not exist");
    }

    let mut resolve = Resolve::default();
    let pkg = resolve.push_dir(&PathBuf::from("wit"))?.0;
    let world = resolve.select_world(pkg, world)?;

    let wasm = componentize_mbt::componentize(&fs::read_to_string(&wat_file)?, resolve, world)?;

    let target = wat_file.with_extension("wasm");
    fs::write(&target, wasm)?;
    println!("Successfully generated: {target:?}");
    Ok(())
}
