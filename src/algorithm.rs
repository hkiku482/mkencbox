use std::{
    io::{Read, Seek, Write},
    path::Path,
};

pub trait AlgorithmRead: Read + Seek {}
pub trait AlgorithmWrite: Write + Seek {}

pub trait Crypto {
    fn encrypt(
        &self,
        reader: &mut dyn AlgorithmRead,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn decrypt(
        &self,
        reader: &mut dyn AlgorithmRead,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Pack {
    fn compression(
        &self,
        in_path: &Path,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn decompression(
        &self,
        reader: &mut dyn AlgorithmRead,
        out_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
