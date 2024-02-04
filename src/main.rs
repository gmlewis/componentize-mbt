use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use wit_bindgen_core::Files;
use wit_parser::{Resolve, UnresolvedPackage, WorldId};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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

    #[clap(value_name = "DOCUMENT", index = 1)]
    wit: PathBuf,

    #[clap(short, long)]
    world: Option<String>,
}

impl Common {
    fn parse_wit(&self) -> Result<(Resolve, WorldId)> {
        let mut resolve = Resolve::default();
        let pkg = if self.wit.is_dir() {
            resolve.push_dir(&self.wit)?.0
        } else {
            resolve.push(UnresolvedPackage::parse_file(&self.wit)?)?
        };
        let world = resolve.select_world(pkg, self.world.as_deref())?;
        Ok((resolve, world))
    }
}

fn main() -> Result<()> {
    match Opt::parse() {
        Opt::Bindgen { opts, args} => {
            let mut files = Files::default();
            let (resolve, world) = args.parse_wit()?;
            opts.build().generate(&resolve, world, &mut files)?;
            for (name, contents) in files.iter() {
                let dst = match &args.out_dir {
                    Some(ref path) => path.join(name),
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
        Opt::Componentize { opts, args } => {
            let (resolve, world) = args.parse_wit()?;
            opts.run(resolve, world, args.out_dir)?;
        }
    }

    Ok(())
}
