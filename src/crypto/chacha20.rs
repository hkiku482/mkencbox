use std::path::PathBuf;

use aes::cipher::{KeyIvInit, StreamCipher};
use anyhow::Result;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use crate::{key_file_phrase, Crypto};

pub struct Chacha20 {
    salt: Option<String>,
    key_filepath: PathBuf,
}

const ITER: u32 = 1_000_000;
const BUFFER_SIZE: usize = 8192;

impl Chacha20 {
    pub fn new(salt: Option<String>, key_filepath: impl Into<PathBuf>) -> Self {
        Self {
            salt,
            key_filepath: key_filepath.into(),
        }
    }

    fn process_contrast(
        &self,
        reader: &mut dyn crate::AlgorithmRead,
        writer: &mut dyn crate::AlgorithmWrite,
    ) -> Result<()> {
        let pass = key_file_phrase(&self.key_filepath)?;
        let salt = match &self.salt {
            Some(v) => v.as_bytes().to_vec(),
            None => vec![],
        };
        let mut base = [0u8; 32 + 12];
        pbkdf2_hmac::<Sha256>(&pass, &salt, ITER, &mut base);
        let key: [u8; 32] = base[..32].try_into()?;
        let nonce: [u8; 12] = base[32..44].try_into()?;

        let mut cipher = chacha20::ChaCha20::new(&key.into(), &nonce.into());
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let read = reader.read(&mut buffer)?;
            cipher.apply_keystream(&mut buffer[..read]);
            writer.write_all(&buffer[..read])?;
            if read < BUFFER_SIZE {
                break;
            }
        }

        Ok(())
    }
}

impl Crypto for Chacha20 {
    fn encrypt(
        &self,
        reader: &mut dyn crate::AlgorithmRead,
        writer: &mut dyn crate::AlgorithmWrite,
    ) -> Result<()> {
        self.process_contrast(reader, writer)
    }

    fn decrypt(
        &self,
        reader: &mut dyn crate::AlgorithmRead,
        writer: &mut dyn crate::AlgorithmWrite,
    ) -> Result<()> {
        self.process_contrast(reader, writer)
    }
}
