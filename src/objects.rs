use std::fmt;
use std::fs;
use std::path::Path;

use std::io;
use std::io::prelude::*;

use anyhow::Context;
use flate2::read::ZlibDecoder;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ObjectType {
    Blob,
    Tree,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blob => f.write_str("blob"),
            Self::Tree => f.write_str("tree"),
        }
    }
}

impl TryFrom<&[u8]> for ObjectType {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"blob" => Ok(ObjectType::Blob),
            b"tree" => Ok(ObjectType::Tree),
            _ => anyhow::bail!(
                "invalid object type {}",
                std::str::from_utf8(value).context("converting object type to utf8")?
            ),
        }
    }
}

impl ObjectType {
    /// Parses a git object and returns a reader for its decompressed content.
    /// The object_name doesn't have to be 20 bytes exact; but it has to be unique.
    pub(crate) fn get_handle(
        object_name: String,
    ) -> anyhow::Result<(ObjectType, impl io::BufRead)> {
        let obj_file = find_one(&object_name)?;

        let file = fs::File::open(obj_file).context("open in .git/objects")?;
        let reader = ZlibDecoder::new(file);
        let mut reader = io::BufReader::new(reader);

        let mut buffer = Vec::new();
        reader
            .read_until(b" "[0], &mut buffer)
            .context("reading object type")?;
        buffer.pop(); // removing the space
        let obj_type = ObjectType::try_from(&buffer[..])?;

        let mut buffer = Vec::new();
        reader
            .read_until(0, &mut buffer)
            .context("reading object file")?;
        buffer.pop(); // removing the null byte

        let size = std::str::from_utf8(&buffer[..])
            .context("parsing size")?
            .parse::<u64>()
            .context("parsing size as u64")?;

        let reader = reader.take(size);

        Ok((obj_type, reader))
    }
}

/// Search git objects based on the **possibly partial** object hash.
fn find_any(hash: &str) -> anyhow::Result<Vec<fs::DirEntry>> {
    let objects_dir = Path::new(".git/objects");
    let dir = fs::read_dir(objects_dir.join(&hash[..2])).context("reading object directory")?;

    let matched_objects: Vec<_> = dir
        .filter(|v| {
            v.as_ref().map_or(false, |de| {
                de.file_name()
                    .to_str()
                    .expect("git objects are sha1 hashes")
                    .to_owned()
                    .starts_with(&hash[2..])
            })
        })
        .map(|v| v.unwrap())
        .collect();

    Ok(matched_objects)
}

/// Get exactly 1 object based on the parial hash passed
fn find_one(hash: &str) -> anyhow::Result<std::path::PathBuf> {
    let matched_objects = find_any(&hash)?;

    let total_matched = matched_objects.len();

    match total_matched {
        1 => Ok(matched_objects[0].path()),
        0 => {
            anyhow::bail!("Object not found {}", hash);
        }
        _ => {
            let objs: Vec<_> = matched_objects
                .iter()
                .map(|v| String::from(v.path().to_str().expect("valid sha1 hashes")))
                .collect();
            anyhow::bail!(
                "Found {} objects with the same name: \n{}",
                objs.len(),
                objs.join("\n")
            )
        }
    }
}
