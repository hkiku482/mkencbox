use std::{
    io::{Read, Seek, Write},
    path::Path,
};

use anyhow::Result;

pub trait AlgorithmRead: Read + Seek {}
impl<T: Read + Seek> AlgorithmRead for T {}
pub trait AlgorithmWrite: Write + Seek {}
impl<T: Write + Seek> AlgorithmWrite for T {}

pub trait Crypto: Send + Sync {
    fn encrypt(
        &self,
        reader: &mut dyn AlgorithmRead,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<()>;
    fn decrypt(
        &self,
        reader: &mut dyn AlgorithmRead,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<()>;
}

pub trait Pack: Send + Sync {
    fn compression(&self, in_path: &Path, writer: &mut dyn AlgorithmWrite) -> Result<()>;
    fn decompression(&self, reader: &mut dyn AlgorithmRead, out_path: &Path) -> Result<()>;
}
