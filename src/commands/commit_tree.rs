use std::io::prelude::*;
use std::fs;
use std::path::Path;

use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use nanoid::nanoid;

use crate::hash_writer::HashWriter;
use crate::CommitTree;

pub(crate) fn handler(input: CommitTree) -> anyhow::Result<String> {

    let root_tree = input.tree_hash;
    let parent = input.parent;
    let mut buffer: Vec<u8> = Vec::new();

    writeln!(&mut buffer, "tree {}", root_tree).context("writing tree in commit")?;

    if let Some(parent) = parent {
        writeln!(&mut buffer, "parent {parent}").context("writing tree in commit")?;
    }

    writeln!(&mut buffer, "author Pravat Pandey <pandeypravat@gmail.com> 1716982150 +0545")
        .context("writing author")?;
    writeln!(&mut buffer, "committer Pravat Pandey <pandeypravat@gmail.com> 1716982150 +0545")
        .context("writing committer")?;
    writeln!(&mut buffer, "").context("writing empty line")?;
    writeln!(&mut buffer, "{}", input.message).context("writing message")?;
    
    let temp_filename = format!("/tmp/{}", nanoid!());
    let mut temp_file = fs::File::create(&Path::new(&temp_filename))
        .with_context(|| format!("creating temporary object file {}", temp_filename))?;

    let writer = ZlibEncoder::new(&mut temp_file, Compression::default());
    let mut writer = HashWriter {
        writer,
        hasher: Sha1::new(),
    };

    write!(&mut writer, "commit {}\0", buffer.len()).context("writing header")?;
    writer.write_all(&buffer[..]).context("writing content")?;


    let obj_hash = writer.hasher.finalize();
    let obj_hash = hex::encode(obj_hash);

    let path = Path::new(".git/objects")
        .join(&obj_hash[..2])
        .join(&obj_hash[2..]);

    fs::create_dir_all(path.parent().unwrap()).with_context(|| {
        format!(
            "create directory for the commit {}",
            path.parent().unwrap().to_string_lossy()
        )
    })?;

    fs::rename(temp_filename, path).context("rename tmp file")?;


    Ok(obj_hash)
}
