use std::{fs::read, path::PathBuf};

use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

pub fn passphrase_from_kfile(kfile: &PathBuf) -> String {
    let bytes = read(kfile).unwrap();
    let sha256sum = sha256::digest(&bytes);
    let md5sum = md5::compute(&bytes);
    let mut p = String::from(sha256sum);
    p.push('0');
    p.push_str(&format!("{:?}", md5sum));
    p
}

pub fn gen_key(password: &str, salt: &[u8], iter: u32) -> ([u8; 32], [u8; 16]) {
    let mut key = [0u8; 48];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iter, &mut key);
    let iv: [u8; 16] = key[32..].try_into().unwrap();
    let key: [u8; 32] = key[..32].try_into().unwrap();
    (key, iv)
}

#[cfg(test)]
mod test {
    use super::passphrase_from_kfile;
    use std::env;

    #[test]
    fn passphrase_from_kfile_test() {
        let mut kfile = env::current_dir().unwrap();
        kfile.push("test/kfile.jpg");
        println!("cd {:?}", kfile);
        let passphrase = passphrase_from_kfile(&kfile);
        assert_eq!(passphrase, "780609d055784ba516d8f989d2c97d4dd1cc149fe2313cfbd679188cc01d3083097ef0732f395021c1fe71f8d6456e693");
    }
}
