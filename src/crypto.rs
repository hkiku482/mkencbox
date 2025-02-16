use crate::error::{Error, ErrorKind};

mod chacha20;

use anyhow::Result;
pub use chacha20::*;

pub fn key_file_phrase(kfile: &std::path::Path) -> Result<Vec<u8>> {
    if !kfile.is_file() {
        return Err(Box::new(Error::from(ErrorKind::InvalidKeyfile)).into());
    }
    let bytes = std::fs::read(kfile)?;
    let sha256sum = sha256::digest(&bytes);
    let md5sum = md5::compute(&bytes);
    let mut p = sha256sum;
    p.push('0');
    p.push_str(&format!("{:?}", md5sum));
    Ok(Vec::from(p.as_bytes()))
}
