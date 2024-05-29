use anyhow::Context;
use std::io;

use crate::objects::ObjectType;

pub(crate) fn handler(input: crate::CatFile) -> anyhow::Result<()> {
    let (obj_type, mut reader) =
        ObjectType::get_handle(input.object_name).context("parse object type")?;

    match obj_type {
        ObjectType::Blob => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            io::copy(&mut reader, &mut handle).context("writing contents to stdout")?;
        }
        _ => {
            anyhow::bail!("invalid object type {:?}", obj_type);
        }
    }
    Ok(())
}
