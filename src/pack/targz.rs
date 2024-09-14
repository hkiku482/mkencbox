use std::{
    fs::{read, read_dir, remove_dir, write, File},
    path::Path,
};

use crate::algorithm;

pub struct TarGz;

impl TarGz {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TarGz {
    fn default() -> Self {
        Self::new()
    }
}

impl algorithm::Pack for TarGz {
    fn compression(&self, in_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if in_path.is_file() {
            // For backwards Compatibility.
            // Don't make *.tar.gz
            return Ok(read(in_path)?);
        }

        let mut tar = tar::Builder::new(Vec::<u8>::new());
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
        Ok(tar.get_mut().to_vec())
    }

    fn decompression(
        &self,
        compressed: &[u8],
        out_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tar = tar::Archive::new(compressed);
        match tar.unpack(out_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                let mut tar = tar::Archive::new(compressed);
                let a = tar.entries().unwrap();
                if a.count() == 1 {
                    remove_dir(out_path)?;
                    Ok(write(out_path, compressed)?)
                } else {
                    Err(Box::new(e))
                }
            }
        }
        // let a = tar.entries().unwrap();
        // if a.count() == 1 {
        //     Ok(write(out_path, compressed)?)
        // } else {
        //     Ok(tar.unpack(out_path)?)
        // }
    }
}
