use crate::{
    algorithm::{AlgorithmRead, AlgorithmWrite, Crypto},
    crypto::key_file_phrase,
    error::{Error, ErrorKind},
};
use aes::{
    cipher::{
        block_padding::{Padding, Pkcs7},
        consts::U16,
        generic_array::GenericArray,
        BlockDecryptMut, BlockEncryptMut, KeyIvInit,
    },
    Aes256,
};
use cbc::{Decryptor, Encryptor};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::{io::SeekFrom, path::PathBuf};

fn cipher_enc(
    iv: [u8; 16],
    key: [u8; 32],
    reader: &mut dyn AlgorithmRead,
    writer: &mut dyn AlgorithmWrite,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cipher = Encryptor::<Aes256>::new(&key.into(), &iv.into());
    let mut buffer = [0u8; BLOCK_SIZE];
    loop {
        let read = reader.read(&mut buffer)?;
        if read < BLOCK_SIZE {
            let mut block = GenericArray::<u8, U16>::default();
            block[..read].copy_from_slice(&buffer[..read]);
            Pkcs7::pad(&mut block, read);
            buffer = block.into();
            cipher.encrypt_block_mut(GenericArray::from_mut_slice(&mut buffer));
            writer.write_all(&buffer)?;
            break;
        }
        cipher.encrypt_block_mut(GenericArray::from_mut_slice(&mut buffer));
        writer.write_all(&buffer)?;
    }
    Ok(())
}

fn cipher_dec(
    iv: [u8; 16],
    key: [u8; 32],
    reader: &mut dyn AlgorithmRead,
    writer: &mut dyn AlgorithmWrite,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cipher = Decryptor::<Aes256>::new(&key.into(), &iv.into());
    let mut buffer = [0u8; BLOCK_SIZE];
    let mut prev_buffer = Option::<[u8; BLOCK_SIZE]>::None;

    loop {
        let read = reader.read(&mut buffer)?;
        cipher.decrypt_block_mut(GenericArray::from_mut_slice(&mut buffer));
        if let Some(prev) = prev_buffer {
            if read == 0 {
                let block = GenericArray::<u8, U16>::from(prev);
                let unpadded = match Pkcs7::unpad(&block) {
                    Ok(v) => v,
                    Err(_) => &prev,
                };
                writer.write_all(unpadded)?;
                break;
            }
            writer.write_all(&prev)?;
        }
        prev_buffer = Some(buffer);
    }
    Ok(())
}
pub struct CbcPbkdf2 {
    salt: Option<String>,
    key_filepath: PathBuf,
}

const ITER: u32 = 600_000;
const BLOCK_SIZE: usize = 16;

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

impl Crypto for CbcPbkdf2 {
    fn encrypt(
        &self,
        reader: &mut dyn AlgorithmRead,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let salt = match self.salt.clone().map(hex::decode) {
            Some(s) => s?,
            None => {
                let mut rng = rand::thread_rng();
                let mut s = [0u8; 16];
                s[0] = b'S';
                s[1] = b'a';
                s[2] = b'l';
                s[3] = b't';
                s[4] = b'e';
                s[5] = b'd';
                s[6] = b'_';
                s[7] = b'_';
                rand::Rng::fill(&mut rng, &mut s[8..16]);
                writer.write_all(&s)?;
                Vec::from(&s[8..16])
            }
        };
        let pass = key_file_phrase(&self.key_filepath)?;
        let (iv, key) = self.gen_key(&pass, &salt)?;
        cipher_enc(iv, key, reader, writer)?;
        Ok(())
    }

    fn decrypt(
        &self,
        reader: &mut dyn AlgorithmRead,
        writer: &mut dyn AlgorithmWrite,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let salt = match self.salt.clone() {
            Some(s) => hex::decode(s)?,
            None => {
                let mut salt_buf = [0u8; 16];
                let _ = reader.read(&mut salt_buf)?;
                if &salt_buf[..8] != "Salted__".as_bytes() {
                    return Err(Box::new(Error::from(ErrorKind::DecryptionError)));
                }
                let s = &salt_buf[8..16];
                reader.seek(SeekFrom::Start(16))?;
                s.to_vec()
            }
        };
        let pass = key_file_phrase(&self.key_filepath)?;
        let (iv, key) = self.gen_key(&pass, &salt)?;
        cipher_dec(iv, key, reader, writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::cipher_enc;
    use crate::crypto::cbcpbkdf2::cipher_dec;
    use std::io::{Read, Seek, Write};
    use tempfile::tempfile;

    #[test]
    fn padded_short() {
        let iv = [1u8; 16];
        let key = [1u8; 32];
        let mut source = tempfile().unwrap();
        let txt = "123456789012345".as_bytes();
        source.write_all(txt).unwrap();
        source.rewind().unwrap();
        let mut encrypted = tempfile().unwrap();

        cipher_enc(iv, key, &mut source, &mut encrypted).unwrap();
        encrypted.rewind().unwrap();
        let mut buf = Vec::new();
        encrypted.read_to_end(&mut buf).unwrap();
        assert_ne!(txt.to_vec(), buf);

        encrypted.rewind().unwrap();
        let mut decrypted = tempfile().unwrap();
        cipher_dec(iv, key, &mut encrypted, &mut decrypted).unwrap();

        source.rewind().unwrap();
        let mut s = Vec::new();
        source.read_to_end(&mut s).unwrap();

        decrypted.rewind().unwrap();
        let mut d = Vec::new();
        decrypted.read_to_end(&mut d).unwrap();

        assert_eq!(s, d)
    }

    #[test]
    fn padded_long() {
        let iv = [1u8; 16];
        let key = [1u8; 32];
        let mut source = tempfile().unwrap();
        let txt = "1234567890123456789012345678901234567890".as_bytes();
        source.write_all(txt).unwrap();
        source.rewind().unwrap();
        let mut encrypted = tempfile().unwrap();

        cipher_enc(iv, key, &mut source, &mut encrypted).unwrap();
        encrypted.rewind().unwrap();
        let mut buf = Vec::new();
        encrypted.read_to_end(&mut buf).unwrap();
        assert_ne!(txt.to_vec(), buf);

        encrypted.rewind().unwrap();
        let mut decrypted = tempfile().unwrap();
        cipher_dec(iv, key, &mut encrypted, &mut decrypted).unwrap();

        source.rewind().unwrap();
        let mut s = Vec::new();
        source.read_to_end(&mut s).unwrap();

        decrypted.rewind().unwrap();
        let mut d = Vec::new();
        decrypted.read_to_end(&mut d).unwrap();

        assert_eq!(s, d)
    }

    #[test]
    fn unpadded() {
        let iv = [1u8; 16];
        let key = [1u8; 32];
        let mut source = tempfile().unwrap();
        let txt = "1234567890123456".as_bytes();
        source.write_all(txt).unwrap();
        source.rewind().unwrap();
        let mut encrypted = tempfile().unwrap();

        cipher_enc(iv, key, &mut source, &mut encrypted).unwrap();
        encrypted.rewind().unwrap();
        let mut buf = Vec::new();
        encrypted.read_to_end(&mut buf).unwrap();
        assert_ne!(txt.to_vec(), buf);

        encrypted.rewind().unwrap();
        let mut decrypted = tempfile().unwrap();
        cipher_dec(iv, key, &mut encrypted, &mut decrypted).unwrap();

        source.rewind().unwrap();
        let mut s = Vec::new();
        source.read_to_end(&mut s).unwrap();

        decrypted.rewind().unwrap();
        let mut d = Vec::new();
        decrypted.read_to_end(&mut d).unwrap();

        assert_eq!(s, d)
    }

    #[test]
    fn unpadded_long() {
        let iv = [1u8; 16];
        let key = [1u8; 32];
        let mut source = tempfile().unwrap();
        let txt = [10u8; 320].to_vec();
        source.write_all(&txt).unwrap();
        source.rewind().unwrap();
        let mut encrypted = tempfile().unwrap();

        cipher_enc(iv, key, &mut source, &mut encrypted).unwrap();
        encrypted.rewind().unwrap();
        let mut buf = Vec::new();
        encrypted.read_to_end(&mut buf).unwrap();
        assert_ne!(txt.to_vec(), buf);
        assert_eq!(buf.len() % 16, 0);

        encrypted.rewind().unwrap();
        let mut decrypted = tempfile().unwrap();
        cipher_dec(iv, key, &mut encrypted, &mut decrypted).unwrap();

        source.rewind().unwrap();
        let mut s = Vec::new();
        source.read_to_end(&mut s).unwrap();

        decrypted.rewind().unwrap();
        let mut d = Vec::new();
        decrypted.read_to_end(&mut d).unwrap();

        assert_eq!(s.len(), d.len());
        assert_eq!(s, d);
    }
}
