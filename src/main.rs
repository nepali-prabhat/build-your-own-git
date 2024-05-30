use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub(crate) mod commands;
pub(crate) mod objects;
pub(crate) mod hash_writer;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Initialize git repository
    Init,

    /// Print the contents of a git object
    CatFile(CatFile),

    /// Creates an object hash of a file
    HashObject(HashObject),

    LsTree(LsTree),

    WriteTree,
}

#[derive(Debug, Parser)]
pub(crate) struct CatFile {
    #[arg(short)]
    pretty_print: bool,
    object_name: String,
}

#[derive(Debug, Parser)]
pub(crate) struct HashObject {
    #[arg(short)]
    write: bool,
    file_path: std::path::PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct LsTree {
    #[arg(long)]
    name_only: bool,
    tree_sha: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            commands::init::handler()?;
        }
        Commands::CatFile(v) => {
            commands::cat_file::handler(v)?;
        }

        Commands::HashObject(v) => {
            let hash = commands::hash_object::handler(v)?;
            println!("{hash}");
        }
        Commands::LsTree(v) => {
            commands::ls_tree::handler(v)?;
        }
        Commands::WriteTree => {
            let hash = commands::write_tree::handler(&PathBuf::from("."))?;
            println!("{hash}");
        }
    }
    Ok(())
}
