use std::fs;

use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use nanoid::nanoid;
use std::io;
use std::io::prelude::*;

use clap::{Parser, Subcommand};
use std::path::Path;

use hex;
use sha1::{digest::core_api::CoreWrapper, Digest, Sha1, Sha1Core};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize git repository
    Init,

    /// Print the contents of a git object
    CatFile {
        #[clap(short)]
        pretty_print: bool,
        object_name: String,
    },

    HashObject {
        #[arg(short)]
        write: bool,
        file_path: std::path::PathBuf,
    },
}

#[derive(Debug, PartialEq)]
enum ObjectType {
    Blob,
}

impl ObjectType {
    fn try_from(s: &str) -> Result<ObjectType, anyhow::Error> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            _ => anyhow::bail!("invalid object type {}", s),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            let _ = fs::create_dir(".git");
            let _ = fs::create_dir(".git/objects");
            let _ = fs::create_dir(".git/refs");
            let head_path = Path::new(".git/HEAD");
            let make_file = head_path.try_exists().map_or(false, |v| !v);
            if make_file {
                fs::write(".git/HEAD", "ref: refs/heads/main\n").context("creating HEAD file")?;
            }
        }
        Commands::CatFile {
            pretty_print: _,
            object_name,
        } => {
            let objects_dir = Path::new(".git/objects");
            let dir = fs::read_dir(objects_dir.join(&object_name[..2]))
                .context("reading object directory")?;

            let matched_objects: Vec<_> = dir
                .filter(|v| {
                    v.as_ref().map_or(false, |de| {
                        de.file_name()
                            .to_str()
                            .expect("git objects are sha1 hashes")
                            .to_owned()
                            .starts_with(&object_name[2..])
                    })
                })
                .map(|v| v.unwrap())
                .collect();

            let total_matched = matched_objects.len();

            if total_matched == 0 {
                anyhow::bail!("git object {} not found", object_name);
            }
            if total_matched > 1 {
                let objs: Vec<_> = matched_objects
                    .iter()
                    .map(|v| String::from(v.path().to_str().expect("valid sha1 hashes")))
                    .collect();
                anyhow::bail!(
                    "Found {} objects with the same name: \n{}",
                    objs.len(),
                    objs.join("\n")
                );
            }

            let file = fs::File::open(matched_objects[0].path()).context("open in .git/objects")?;
            let reader = ZlibDecoder::new(&file);
            let mut reader = io::BufReader::new(reader);

            let mut buffer = Vec::new();

            reader
                .read_until(0, &mut buffer)
                .context("reading object file")?;
            buffer.pop(); // removing the null byte

            let s = std::str::from_utf8(&buffer).expect("must be valid utf8");
            let mut s = s.split(" ");

            let ot = s.next().context("no object type")?;
            let obj_type = ObjectType::try_from(ot)?;

            match obj_type {
                ObjectType::Blob => {
                    let n = s.next().context("reading number of bytes")?;
                    let n = n.parse::<usize>().context("parsing number of bytes")?;

                    // read exactly n number of bytes from the reader;
                    buffer.clear();
                    buffer.resize(n, 0);

                    reader
                        .read_exact(&mut buffer)
                        .context("reading the contents of the file")?;
                    let x = reader.read(&mut [0]).expect("ensuring EOF");
                    anyhow::ensure!(x == 0, "didn't reach EOF");

                    // read exactly n number of bytes from the reader;
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(&buffer)?;
                }
            }
        }

        Commands::HashObject { write, file_path } => {
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

            println!("{}", obj_hash);
        }
    }
    Ok(())
}

struct HashWriter<W> {
    writer: W,
    hasher: CoreWrapper<Sha1Core>,
}

impl<W: Write> Write for HashWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(&buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
