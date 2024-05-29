use anyhow::Context;
use std::fs;
use std::path::Path;

pub(crate) fn handler() -> anyhow::Result<()> {
    let _ = fs::create_dir(".git");
    let _ = fs::create_dir(".git/objects");
    let _ = fs::create_dir(".git/refs");
    let head_path = Path::new(".git/HEAD");
    let make_file = head_path.try_exists().map_or(false, |v| !v);
    if make_file {
        fs::write(".git/HEAD", "ref: refs/heads/main\n").context("creating HEAD file")?;
    }
    Ok(())
}
