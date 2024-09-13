use std::io::{Error, Write};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};

pub fn encrypt(
    key: &[u8; 32],
    iv: &[u8; 16],
    source: &[u8],
    salt: Option<&[u8]>,
) -> Result<Vec<u8>, Error> {
    let mut out_buff = Vec::<u8>::new();
    let mut output_buf = vec![0; source.len() + 512].into_boxed_slice();

    let cipher = cbc::Encryptor::<aes::Aes256>::new(key.into(), iv.into());
    let enc = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(source, &mut output_buf)
        .unwrap();

    if let Some(s) = salt {
        let _ = out_buff.write("Salted__".as_bytes());
        let _ = out_buff.write(s);
    }
    let _ = out_buff.write(enc);

    Ok(out_buff)
}

pub fn decrypt(key: &[u8; 32], iv: &[u8; 16], source: &[u8]) -> Result<Vec<u8>, Error> {
    let mut out_buff = Vec::<u8>::new();
    let mut output_buf = vec![0; source.len() + 512].into_boxed_slice();

    let head = std::str::from_utf8(&source[0..8]).unwrap_or("");

    let cipher = cbc::Decryptor::<aes::Aes256>::new(key.into(), iv.into());
    if head == "Salted__" {
        let plain = cipher
            .decrypt_padded_b2b_mut::<Pkcs7>(&source[16..], &mut output_buf)
            .unwrap();
        let _ = out_buff.write(plain).unwrap();
    } else {
        let plain = cipher
            .decrypt_padded_b2b_mut::<Pkcs7>(source, &mut output_buf)
            .unwrap();
        let _ = out_buff.write(plain).unwrap();
    }

    Ok(out_buff)
}

#[cfg(test)]
mod test {
    use super::{decrypt, encrypt};
    use crate::mkencbox::crypto::gen_key;
    use std::{env, fs::read};

    #[test]
    fn encrypt_test() {
        let salt = hex::decode("ab19c9806c103aaf").unwrap();
        let (key, iv) = gen_key("password", &salt, 100000);
        let mut target = env::current_dir().unwrap();
        target.push("test/text.txt");
        let bytes = read(target).unwrap();
        let mut out = env::current_dir().unwrap();
        out.push("test/text.enc");
        encrypt(&key, &iv, &bytes, Some(salt.as_slice())).unwrap();
    }

    #[test]
    fn decrypt_test() {
        let salt = hex::decode("ab19c9806c103aaf").unwrap();
        let (key, iv) = gen_key("password", &salt, 100000);
        let mut target = env::current_dir().unwrap();
        target.push("test/text.enc");
        let mut out = env::current_dir().unwrap();
        out.push("test/text.dec");
        let bytes = read(target).unwrap();
        decrypt(&key, &iv, &bytes).unwrap();
    }
}
