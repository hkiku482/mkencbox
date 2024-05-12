use std::{
    fs::{read, File},
    io::{Error, Read, Write},
    path::PathBuf,
};

use rand::{thread_rng, Rng};

use self::{
    compression::decompression_or_create,
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
    if output.exists() {
        return Err(Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{:?} already exists", output),
        ));
    }

    let data: Vec<u8>;
    if input.is_dir() {
        data = compression::compression_dir(input).unwrap();
    } else {
        data = read(input.clone()).unwrap();
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
    let encrypted = encrypt(&key, &iv, &data, embedded_salt).unwrap();
    let mut f = File::create(output).unwrap();
    f.write_all(&encrypted).unwrap();

    Ok(())
}

pub fn dec(
    kfile: &PathBuf,
    input: &PathBuf,
    output: &PathBuf,
    salt: Option<&Vec<u8>>,
) -> Result<(), Error> {
    if output.exists() {
        return Err(Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{:?} already exists", output),
        ));
    }

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
    let encrypted = read(input).unwrap();
    let decrypted = decrypt(&key, &iv, &encrypted).unwrap();

    decompression_or_create(&decrypted, output).unwrap();
    Ok(())
}
