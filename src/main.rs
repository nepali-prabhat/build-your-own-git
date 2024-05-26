#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

use clap::{arg, Command};

fn cli() -> Command {
    Command::new("git")
        .about("codecrafters git")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("init")
            .about("Initialize repo")
        )
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let matches = cli().get_matches();
    
    match matches.subcommand() {
        Some(("init", _)) => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        }
        _ => unreachable!(),
    }

}
