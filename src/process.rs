use std::{
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
};

use tempfile::{tempfile, NamedTempFile};

use crate::algorithm::{self, AlgorithmRead, AlgorithmWrite};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Enc,
    Dec,
}

impl AlgorithmRead for NamedTempFile {}
impl AlgorithmWrite for NamedTempFile {}
impl AlgorithmRead for File {}
impl AlgorithmWrite for File {}

pub struct Process {
    target: Target,
    pack_algorithm: Box<dyn algorithm::Pack>,
    crypto_algorithm: Box<dyn algorithm::Crypto>,
    from_path: PathBuf,
    to_path: PathBuf,
}

impl Process {
    pub fn new(
        target: Target,
        pack_algorithm: Box<dyn algorithm::Pack>,
        crypto_algorithm: Box<dyn algorithm::Crypto>,
        from_path: impl Into<PathBuf>,
        to_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            target,
            pack_algorithm,
            crypto_algorithm,
            from_path: from_path.into(),
            to_path: to_path.into(),
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.to_path.exists() {
            let e = std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("{:?} already exists", self.to_path),
            );
            return Err(Box::new(e));
        }
        match self.target {
            Target::Enc => {
                self.enc()?;
            }
            Target::Dec => {
                self.dec()?;
            }
        };
        Ok(())
    }

    fn enc(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = NamedTempFile::new()?;
        let mut dst = File::create(&self.to_path)?;

        self.pack_algorithm
            .compression(self.from_path.as_path(), &mut tmp)?;
        tmp.rewind()?;
        println!("packed");
        self.crypto_algorithm.encrypt(&mut tmp, &mut dst)?;
        println!("encrypted");
        Ok(())
    }

    fn dec(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut src = File::open(&self.from_path)?;
        let mut tmp = tempfile()?;

        let mut buf = Vec::new();
        src.read_to_end(&mut buf)?;
        src.rewind()?;

        self.crypto_algorithm.decrypt(&mut src, &mut tmp)?;
        tmp.rewind()?;
        println!("decrypted");
        self.pack_algorithm.decompression(&mut tmp, &self.to_path)?;
        println!("unpacked");
        Ok(())
    }
}
