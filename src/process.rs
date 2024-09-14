use std::{
    fs::{read, write},
    path::PathBuf,
};

use crate::algorithm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Enc,
    Dec,
}

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
        let packed = self.pack_algorithm.compression(&self.from_path)?;
        let encrypted = self.crypto_algorithm.encrypt(&packed)?;
        write(&self.to_path, encrypted)?;
        Ok(())
    }

    fn dec(&self) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted = read(&self.from_path)?;
        let packed = self.crypto_algorithm.decrypt(&encrypted)?;
        self.pack_algorithm.decompression(&packed, &self.to_path)?;
        Ok(())
    }
}
