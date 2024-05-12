use std::{
    fs::{read_dir, remove_dir, File},
    io::{Error, Write},
    path::PathBuf,
};

use tar::{Archive, Builder};

pub fn compression_dir(in_dirpath: &PathBuf) -> Result<Vec<u8>, Error> {
    let mut archive = Builder::new(Vec::<u8>::new());
    for entry in read_dir(in_dirpath)? {
        let e = entry.unwrap();
        let p = &e.path();
        if p.is_dir() {
            archive.append_dir_all(p.file_name().unwrap(), p).unwrap()
        } else if p.is_file() {
            let mut f = File::open(p).unwrap();
            archive.append_file(p.file_name().unwrap(), &mut f).unwrap()
        }
    }
    archive.finish().unwrap();
    Ok(archive.get_mut().to_vec())
}

pub fn decompression_or_create(source: &[u8], out_dirname: &PathBuf) -> Result<(), Error> {
    let mut archive = Archive::new(source);
    match archive.unpack(out_dirname) {
        Ok(_) => Ok(()),
        Err(e) => {
            if out_dirname.exists() && out_dirname.is_dir() {
                let d = read_dir(out_dirname).unwrap();
                if d.count() == 0 {
                    remove_dir(out_dirname).unwrap();
                }
            } else {
                panic!("{}", e);
            }
            let mut file = File::create(out_dirname).unwrap();
            file.write_all(source).unwrap();
            Ok(())
        }
    }
}
