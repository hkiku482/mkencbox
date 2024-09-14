use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use walkdir::WalkDir;

pub fn rs_path() -> PathBuf {
    PathBuf::from("tests/resource")
}

pub fn ws_path(tag: &str) -> PathBuf {
    let mut p = PathBuf::from("tests/workspace");
    p.push(tag);
    p
}

pub fn prepare(tag: &str) {
    let ws = ws_path(tag);
    if ws.exists() {
        remove_dir_all(&ws).unwrap();
    }

    create_dir_all(&ws).unwrap();
}

pub fn relative_path(
    tag: &str,
    infile_relative: &str,
    outfile_relative: &str,
) -> (PathBuf, PathBuf) {
    let mut infile = rs_path();
    infile.push(infile_relative);

    let mut outfile = ws_path(tag);
    outfile.push(outfile_relative);

    (infile, outfile)
}

pub fn kfile() -> PathBuf {
    let mut kfile = rs_path();
    kfile.push("keyfile");
    kfile
}

pub fn dir_entries(base: PathBuf) -> Vec<String> {
    let mut entries = Vec::new();
    for e in WalkDir::new(&base).into_iter().filter_map(|e| e.ok()) {
        let p = e.path();
        if let Ok(relative) = p.strip_prefix(&base) {
            entries.push(relative.to_string_lossy().to_string());
        }
    }
    entries
}
