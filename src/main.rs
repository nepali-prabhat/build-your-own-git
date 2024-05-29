use clap::{Parser, Subcommand};

pub(crate) mod commands;
pub(crate) mod objects;

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
            commands::hash_object::handler(v)?;
        }
        Commands::LsTree(v) => {
            commands::ls_tree::handler(v)?;
        }
    }
    Ok(())
}
