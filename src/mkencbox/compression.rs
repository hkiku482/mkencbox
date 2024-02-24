use std::{
    fs::File,
    io::{Error, ErrorKind},
    path::PathBuf,
};

use tar::{Archive, Builder};

pub fn compression(in_dirpath: &PathBuf, out_filepath: &PathBuf) -> Result<(), Error> {
    if out_filepath.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("{:?} already exists", out_filepath),
        ));
    }

    let mut archive = Builder::new(File::create(out_filepath).unwrap());
    archive
        .append_dir_all(in_dirpath.file_name().unwrap(), in_dirpath)
        .unwrap();
    archive.finish().unwrap();
    Ok(())
}

pub fn decompression(in_filepath: &PathBuf, out_dirname: &PathBuf) -> Result<(), Error> {
    if out_dirname.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("{:?} already exists", out_dirname),
        ));
    }

    let mut archive = Archive::new(File::open(in_filepath).unwrap());
    archive.unpack(out_dirname).unwrap();
    Ok(())
}

// #[cfg(test)]
// mod test {
//     use std::{env, path::PathBuf};

//     use crate::mkencbox::compression::compression;

//     use super::decompression;

//     #[test]
//     fn compression_test() {
//         let cudir = env::current_dir().unwrap();
//         let mut s = String::from(cudir.to_str().unwrap());
//         s.push_str("/test/dir_a");
//         let mut o = String::from(cudir.to_str().unwrap());
//         o.push_str("/test/dir_a.tar.gz");
//         let _ = compression(&PathBuf::from(s), &PathBuf::from(o));
//     }

//     #[test]
//     fn decompression_test() {
//         let cudir = env::current_dir().unwrap();
//         let mut s = String::from(cudir.to_str().unwrap());
//         s.push_str("/test/dir_a.tar.gz");
//         let mut o = String::from(cudir.to_str().unwrap());
//         o.push_str("/test/dir_b");
//         let _ = decompression(&PathBuf::from(s), &PathBuf::from(o));
//     }
// }
