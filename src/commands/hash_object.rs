use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::fs;

use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use nanoid::nanoid;
use sha1::{Digest, Sha1};

use hex;

use crate::hash_writer::HashWriter;


pub(crate) fn handler(inputs: crate::HashObject) -> anyhow::Result<String> {
    let write = inputs.write;
    let file_path = inputs.file_path;

    let num_bytes = fs::metadata(&file_path)
        .context("Reading file metadata")?
        .len();

    // temporary file because we don't know the hash yet
    let temp_filename = format!("/tmp/{}", nanoid!());
    let temp_obj_file = fs::File::create(&Path::new(&temp_filename))
        .with_context(|| format!("creating temporary object file {}", temp_filename))?;

    let writer = ZlibEncoder::new(temp_obj_file, Compression::default());
    let hasher = Sha1::new();
    let mut writer = if write {
        HashWriter::<Box<dyn Write>> {
            writer: Box::new(writer),
            hasher,
        }
    } else {
        HashWriter::<Box<dyn Write>> {
            writer: Box::new(io::sink()),
            hasher,
        }
    };

    write!(writer, "{} {}\0", "blob", num_bytes).context("writing header")?;

    io::copy(
        &mut fs::File::open(file_path).context("opening the file for content")?,
        &mut writer,
    )
    .context("writing contents of file to the temporary file")?;

    let obj_hash = writer.hasher.finalize();
    let obj_hash = hex::encode(obj_hash);

    if write {
        let writer_path = Path::new(".git/objects")
            .join(&obj_hash[0..2])
            .join(&obj_hash[2..]);

        fs::create_dir_all(writer_path.parent().expect("never returns None"))
            .context("creating object directory")?;

        fs::rename(&temp_filename, &writer_path).context("creating object file")?;
    }

    Ok(obj_hash)
}

