use std::path::Path;

pub trait Crypto {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

pub trait Pack {
    fn compression(&self, in_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn decompression(
        &self,
        compressed: &[u8],
        out_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
