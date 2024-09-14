use crate::{
    algorithm,
    crypto::key_file_phrase,
    error::{Error, ErrorKind},
};
use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes256,
};
use cbc::{Decryptor, Encryptor};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::{io::Write, path::PathBuf};

pub struct CbcPbkdf2 {
    salt: Option<String>,
    key_filepath: PathBuf,
}

const ITER: u32 = 600_000;

impl CbcPbkdf2 {
    pub fn new(salt: Option<String>, key_filepath: impl Into<PathBuf>) -> Self {
        Self {
            salt,
            key_filepath: key_filepath.into(),
        }
    }

    fn gen_key(
        &self,
        pass: &[u8],
        salt: &[u8],
    ) -> Result<([u8; 16], [u8; 32]), Box<dyn std::error::Error>> {
        let mut key = [0u8; 48];
        pbkdf2_hmac::<Sha256>(pass, salt, ITER, &mut key);
        let iv: [u8; 16] = key[32..].try_into()?;
        let key: [u8; 32] = key[..32].try_into()?;
        Ok((iv, key))
    }
}

impl algorithm::Crypto for CbcPbkdf2 {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let salt = self.salt.clone();
        let salt = match salt.map(hex::decode) {
            Some(s) => s?,
            None => {
                let mut rng = rand::thread_rng();
                let mut s = [0u8; 8];
                rand::Rng::fill(&mut rng, &mut s);
                Vec::from(s)
            }
        };
        let pass = key_file_phrase(&self.key_filepath)?;
        let (iv, key) = self.gen_key(&pass, &salt)?;

        let cipher = Encryptor::<Aes256>::new(&key.into(), &iv.into());
        let mut buffer = vec![0; data.len() + 512].into_boxed_slice();
        let encrypted = match cipher.encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buffer) {
            Ok(v) => v,
            Err(_) => return Err(Box::new(Error::from(ErrorKind::EncryptionError))),
        };

        if self.salt.is_none() {
            let mut buffer = Vec::new();
            let _ = buffer.write("Salted__".as_bytes())?;
            let _ = buffer.write(&salt)?;
            let _ = buffer.write(encrypted);
            Ok(buffer)
        } else {
            Ok(encrypted.to_vec())
        }
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let salt = self.salt.clone();
        let salt = match salt {
            Some(s) => hex::decode(s)?,
            None => {
                if data.len() < 16 {
                    return Err(Box::new(Error::from(ErrorKind::DecryptionError)));
                }
                let s = &data[8..16];
                s.to_vec()
            }
        };
        let pass = key_file_phrase(&self.key_filepath)?;
        let (iv, key) = self.gen_key(&pass, &salt)?;

        let cipher = Decryptor::<Aes256>::new(&key.into(), &iv.into());

        let embedded_salt = std::str::from_utf8(&data[0..8]).unwrap_or("") == "Salted__";
        let mut buffer = vec![0; data.len() + 512].into_boxed_slice();
        let data = if embedded_salt { &data[16..] } else { data };
        let plain = match cipher.decrypt_padded_b2b_mut::<Pkcs7>(data, &mut buffer) {
            Ok(p) => p,
            Err(_) => {
                return Err(Box::new(Error::from(ErrorKind::DecryptionError)));
            }
        };
        Ok(plain.to_vec())
    }
}
