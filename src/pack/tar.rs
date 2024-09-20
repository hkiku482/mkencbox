use std::{
    fs::{read_dir, remove_dir, File},
    io::{copy, BufReader},
    path::Path,
};

use crate::algorithm::{self, AlgorithmRead, AlgorithmWrite};

pub struct Tar;

impl Tar {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Tar {
    fn default() -> Self {
        Self::new()
    }
}

impl algorithm::Pack for Tar {
    fn compression(
        &self,
        in_path: &Path,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if in_path.is_file() {
            let f = File::open(in_path)?;
            let mut buf_reader = BufReader::new(f);
            let _ = copy(&mut buf_reader, writer)?;
            return Ok(());
        }

        let mut tar = tar::Builder::new(writer);
        for entry in read_dir(in_path)? {
            let entry_path = entry?.path();
            if entry_path.is_file() {
                let mut file = File::open(&entry_path)?;
                tar.append_file(entry_path.file_name().unwrap(), &mut file)?;
            } else if entry_path.is_dir() {
                tar.append_dir_all(entry_path.file_name().unwrap(), &entry_path)?;
            }
        }
        tar.finish()?;
        Ok(())
    }

    fn decompression(
        &self,
        reader: &mut dyn AlgorithmRead,
        out_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tar = tar::Archive::new(reader);
        match tar.unpack(out_path) {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::Other {
                    remove_dir(out_path)?;
                    let mut file = File::create(out_path)?;
                    let reader = tar.into_inner();
                    reader.rewind()?;
                    copy(reader, &mut file)?;
                    Ok(())
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }
}
