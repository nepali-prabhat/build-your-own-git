use std::io;
use std::io::prelude::*;
use sha1::{digest::core_api::CoreWrapper, Digest, Sha1Core};

#[derive(Debug)]
pub(crate) struct HashWriter<W> {
    pub(crate) writer: W,
    pub(crate) hasher: CoreWrapper<Sha1Core>,
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
