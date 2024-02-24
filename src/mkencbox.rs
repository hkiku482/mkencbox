use std::{
    fs::File,
    io::{Error, Read},
    path::PathBuf,
};

use rand::{thread_rng, Rng};

use self::{
    compression::decompression,
    crypto::gen_key,
    process::{decrypt, encrypt},
};

mod compression;
mod crypto;
mod process;

const ITER: u32 = 600_000;

pub fn enc(
    kfile: &PathBuf,
    input: &PathBuf,
    output: &PathBuf,
    salt: Option<&Vec<u8>>,
) -> Result<(), Error> {
    let mut target = input.clone();
    if input.is_dir() {
        target.set_extension("tar.gz");
        compression::compression(input, &target).unwrap();
    }

    let password = crypto::passphrase_from_kfile(kfile);
    let user_defined_salt = salt;
    let salt = match salt {
        Some(s) => {
            if s.len() < 8 {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "salt size too short",
                ));
            }
            if 16 < s.len() {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "salt size too large",
                ));
            }
            Vec::from(s.as_slice())
        }
        None => {
            let mut rng = thread_rng();
            let mut s = [0u8; 8];
            rng.fill(&mut s);
            Vec::from(s)
        }
    };
    let embedded_salt = {
        if user_defined_salt.is_none() {
            Some(salt.as_slice())
        } else {
            None
        }
    };
    let (key, iv) = gen_key(&password, &salt, ITER);
    encrypt(&key, &iv, &target, output, embedded_salt).unwrap();

    Ok(())
}

pub fn dec(
    kfile: &PathBuf,
    input: &PathBuf,
    output: &PathBuf,
    salt: Option<&Vec<u8>>,
) -> Result<(), Error> {
    let password = crypto::passphrase_from_kfile(kfile);
    let salt = match salt {
        Some(s) => Vec::from(s.as_slice()),
        None => {
            let mut f = File::open(input).unwrap();
            let mut b = [0; 16];
            f.read_exact(&mut b).unwrap();
            let s: [u8; 8] = b[8..].try_into().unwrap();
            Vec::from(s)
        }
    };
    let (key, iv) = gen_key(&password, &salt, ITER);
    decrypt(&key, &iv, input, output).unwrap();

    if output.try_exists().unwrap() {
        let output_name = output.clone();
        let filename = output_name.file_name().unwrap().to_str().unwrap();
        if filename.contains(".tar.gz") {
            let p = output.parent().unwrap();
            let mut p = p.to_path_buf();
            if let Some(index) = filename.rfind(".tar.gz") {
                p.push(filename[..index].to_string());
            }
            decompression(&output, &p).unwrap();
        }
    }
    Ok(())
}
