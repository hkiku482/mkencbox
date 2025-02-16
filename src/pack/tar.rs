use std::{
    fs::{read_dir, remove_dir, File},
    io::{copy, BufReader},
    path::Path,
};

use anyhow::Result;

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
    fn compression(&self, in_path: &Path, writer: &mut dyn AlgorithmWrite) -> Result<()> {
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

    fn decompression(&self, reader: &mut dyn AlgorithmRead, out_path: &Path) -> Result<()> {
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
                    Err(e.into())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Pack;

    use super::Tar;
    use std::fs::{self, create_dir, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn dir_compression_and_decompression_test() {
        let packer = Tar;

        let origin_dir = TempDir::new().unwrap();
        let dir_path = origin_dir.path();

        let file_paths = ["file1", "file2", "file3"];
        for file_name in &file_paths {
            let file_path = dir_path.join(file_name);
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "{file_name}").unwrap();
        }
        let depth_dir = dir_path.join("directory1");
        let _ = create_dir(&depth_dir);
        let mut file = File::create(depth_dir.join("file4")).unwrap();
        writeln!(file, "file4").unwrap();

        let mut comp_to = NamedTempFile::new().unwrap();

        packer.compression(dir_path, &mut comp_to).unwrap();

        let packer = Tar;
        let mut reader = File::open(comp_to.path()).unwrap();
        let out_dir = TempDir::new().unwrap();
        let _ = packer.decompression(&mut reader, out_dir.path());

        assert!(compare_dirs(origin_dir.path(), out_dir.path()))
    }

    fn compare_dirs(dir1: &Path, dir2: &Path) -> bool {
        let entries1 = get_dir_entries(dir1);
        let entries2 = get_dir_entries(dir2);

        if entries1 != entries2 {
            return false;
        }

        for entry in &entries1 {
            let path1 = dir1.join(entry);
            let path2 = dir2.join(entry);

            if path1.is_dir() && path2.is_dir() {
                if !compare_dirs(&path1, &path2) {
                    return false;
                }
            } else if path1.is_file() && path2.is_file() {
                let meta1 = fs::metadata(&path1).unwrap();
                let meta2 = fs::metadata(&path2).unwrap();
                if meta1.len() != meta2.len() {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn get_dir_entries(dir: &Path) -> Vec<String> {
        let mut entries: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        entries.sort();
        entries
    }
}
