mod build;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use wit_bindgen_core::Files;
use wit_parser::{Resolve, UnresolvedPackage};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum Opt {
    BindGen {
        #[clap(flatten)]
        opts: wit_bindgen_mbt::Opts,
        #[clap(flatten)]
        args: Common,
    },
    Build {
        #[clap(flatten)]
        opts: build::Opts,
    },
}

#[derive(Debug, Parser)]
struct Common {
    #[clap(long = "out-dir")]
    out_dir: Option<PathBuf>,

    #[clap(value_name = "DOCUMENT", index = 1)]
    wit: PathBuf,

    #[clap(short, long)]
    world: Option<String>,
}

fn main() -> Result<()> {
    match Opt::parse() {
        Opt::BindGen { opts, args } => {
            let mut files = Files::default();
            let mut resolve = Resolve::default();
            let pkg = if args.wit.is_dir() {
                resolve.push_dir(&args.wit)?.0
            } else {
                resolve.push(UnresolvedPackage::parse_file(&args.wit)?)?
            };
            let world = resolve.select_world(pkg, args.world.as_deref())?;
            opts.build().generate(&resolve, world, &mut files)?;
            for (name, contents) in files.iter() {
                let dst = match &args.out_dir {
                    Some(path) => path.join(name),
                    None => name.into(),
                };
                println!("Generating {:?}", dst);
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create {:?}", parent))?;
                }
                std::fs::write(&dst, contents)
                    .with_context(|| format!("failed to write {:?}", dst))?;
            }
        }
        Opt::Build { opts } => {
            opts.run()?;
        }
    }

    Ok(())
}
