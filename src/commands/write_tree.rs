use anyhow::Context;
use is_executable::IsExecutable;
use nanoid::nanoid;
use sha1::{Digest, Sha1};
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::commands::hash_object;
use crate::hash_writer::HashWriter;
use crate::HashObject;

pub(crate) fn handler(root: &Path) -> anyhow::Result<String> {
    let entries = fs::read_dir(&root).context("reading the current directory")?;
    let mut entries = entries
        .map(|v| v.map(|v| v.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .context("reading cd's paths")?;
    entries.sort();

    let temp_filename = format!("/tmp/{}", nanoid!());
    let mut temp_file = fs::File::create(&temp_filename)
        .with_context(|| format!("create temporary file for {:?}", &root))?;

    // todo: create a temporary file

    let mut buffer = Vec::new();
    for e in entries {
        let name = e
            .file_name()
            .with_context(|| format!("reading filename {:?}", e))?;

        if e.is_dir() && name == ".git" {
            continue;
        }

        let mode = if e.is_file() {
            if e.is_executable() {
                "100755"
            } else if e.is_symlink() {
                "120000"
            } else {
                "100644"
            }
        } else {
            "40000"
        };

        let hash = if e.is_file() {
            hash_object::handler(HashObject {
                write: true,
                file_path: e.clone(),
            })
            .with_context(|| format!("creating hash object"))?
        } else {
            handler(&e).with_context(|| format!("creating hash object"))?
        };
        let hash = hex::decode(hash).context("decode hash")?;

        write!(&mut buffer, "{} {}\0", mode, name.to_string_lossy()).context("writing mode")?;
        buffer
            .write_all(&hash[..])
            .context("writing un-encoded hash to buffer")?;

        temp_file
            .write_all(&buffer[..])
            .context("copying buffer to file")?;

        buffer.clear();
    }

    let size = temp_file
        .metadata()
        .with_context(|| format!("reading metadata of temp file {}", &temp_filename))?
        .len();

    let temp_filename_encoded = format!("/tmp/{}", nanoid!());
    let temp_file_encoded = fs::File::create(&temp_filename_encoded)
        .with_context(|| format!("create (encoded) temporary file for {:?}", &root))?;

    let writer = ZlibEncoder::new(temp_file_encoded, Compression::default());
    let mut writer = HashWriter {
        writer,
        hasher: Sha1::new(),
    };

    write!(writer, "{} {}\0", "tree", size).context("writing header")?;

    io::copy(
        &mut fs::File::open(&temp_filename)
            .with_context(|| format!("can't open file {}", &temp_filename))?,
        &mut writer,
    )
    .with_context(|| {
        format!(
            "copying contents of {} to the encoded file {}",
            &temp_filename, &temp_filename_encoded
        )
    })?;

    let obj_hash = writer.hasher.finalize();
    let obj_hash = hex::encode(obj_hash);

    let path = Path::new(".git/objects")
        .join(&obj_hash[..2])
        .join(&obj_hash[2..]);

    fs::create_dir_all(path.parent().unwrap()).with_context(|| {
        format!(
            "create directory for the tree {}: {}",
            &root.to_string_lossy(),
            path.parent().unwrap().to_string_lossy()
        )
    })?;

    fs::rename(temp_filename_encoded, path).context("move file to objects directory")?;
    fs::remove_file(temp_filename).context("deleting uncompressed tree file")?;

    return Ok(obj_hash);
}
