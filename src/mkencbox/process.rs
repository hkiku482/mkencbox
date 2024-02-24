use std::{
    fs::{read, File},
    io::{Error, Write},
    path::PathBuf,
};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};

pub fn encrypt(
    key: &[u8; 32],
    iv: &[u8; 16],
    target_filename: &PathBuf,
    output_filename: &PathBuf,
    salt: Option<&[u8]>,
) -> Result<(), Error> {
    let target_file = read(target_filename).unwrap();

    if output_filename.exists() {
        return Err(Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{:?} already exists", output_filename),
        ));
    }
    let mut output_file = File::create(output_filename).unwrap();
    let mut output_buf = vec![0; target_file.len() + 512].into_boxed_slice();

    let cipher = cbc::Encryptor::<aes::Aes256>::new(key.into(), iv.into());
    let enc = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(&target_file, &mut output_buf)
        .unwrap();

    if let Some(s) = salt {
        output_file.write_all("Salted__".as_bytes()).unwrap();
        output_file.write_all(s).unwrap();
    }
    output_file.write_all(enc).unwrap();

    Ok(())
}

pub fn decrypt(
    key: &[u8; 32],
    iv: &[u8; 16],
    encrypt_filename: &PathBuf,
    output_filename: &PathBuf,
) -> Result<(), Error> {
    let encrypt_file = read(encrypt_filename).unwrap();

    if output_filename.exists() {
        return Err(Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{:?} already exists", output_filename),
        ));
    }
    let mut output_file = File::create(output_filename).unwrap();
    let mut output_buf = vec![0; encrypt_file.len() + 512].into_boxed_slice();

    let head = match std::str::from_utf8(&encrypt_file[0..8]) {
        Ok(s) => s,
        Err(_) => "",
    };

    let cipher = cbc::Decryptor::<aes::Aes256>::new(key.into(), iv.into());
    if head == "Salted__" {
        let plain = cipher
            .decrypt_padded_b2b_mut::<Pkcs7>(&encrypt_file[16..], &mut output_buf)
            .unwrap();
        output_file.write_all(plain).unwrap();
    } else {
        let plain = cipher
            .decrypt_padded_b2b_mut::<Pkcs7>(&encrypt_file, &mut output_buf)
            .unwrap();
        output_file.write_all(plain).unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::{decrypt, encrypt};
    use crate::mkencbox::crypto::gen_key;
    use std::env;

    #[test]
    fn encrypt_test() {
        let salt = hex::decode("ab19c9806c103aaf").unwrap();
        let (key, iv) = gen_key("password", &salt, 100000);
        let mut target = env::current_dir().unwrap();
        target.push("test/text.txt");
        let mut out = env::current_dir().unwrap();
        out.push("test/text.enc");
        encrypt(
            &key,
            &iv,
            &target,
            &out,
            Some(salt.as_slice().try_into().unwrap()),
        )
        .unwrap();
    }

    #[test]
    fn decrypt_test() {
        let salt = hex::decode("ab19c9806c103aaf").unwrap();
        let (key, iv) = gen_key("password", &salt, 100000);
        let mut target = env::current_dir().unwrap();
        target.push("test/text.enc");
        let mut out = env::current_dir().unwrap();
        out.push("test/text.dec");
        decrypt(&key, &iv, &target, &out).unwrap();
    }
}
