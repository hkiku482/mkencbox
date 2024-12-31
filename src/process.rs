use std::{
    fs::File,
    io::{BufReader, BufWriter, Seek},
    path::PathBuf,
};

use tempfile::{tempfile, NamedTempFile};

use crate::{Crypto, Pack};

const CAPACITY: usize = 8 * 1024 * 1024; // 8MiB

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Enc,
    Dec,
}

pub struct Process {
    target: Target,
    pack_algorithm: Box<dyn Pack>,
    crypto_algorithm: Box<dyn Crypto>,

    from_path: PathBuf,
    to_path: PathBuf,
}

impl Process {
    pub fn new(
        target: Target,
        pack_algorithm: Box<dyn Pack>,
        crypto_algorithm: Box<dyn Crypto>,
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
        let tmp = NamedTempFile::new()?;
        let dst = File::create(&self.to_path)?;

        let mut writer = BufWriter::with_capacity(CAPACITY, tmp);

        self.pack_algorithm
            .compression(self.from_path.as_path(), &mut writer)?;
        println!("packed");

        let mut tmp = writer.into_inner()?;
        tmp.rewind()?;

        let mut reader = BufReader::with_capacity(CAPACITY, tmp);
        let mut writer = BufWriter::with_capacity(CAPACITY, dst);

        self.crypto_algorithm.encrypt(&mut reader, &mut writer)?;
        println!("encrypted");

        Ok(())
    }

    fn dec(&self) -> Result<(), Box<dyn std::error::Error>> {
        let src = File::open(&self.from_path)?;
        let tmp = tempfile()?;

        let mut reader = BufReader::with_capacity(CAPACITY, src);
        let mut writer = BufWriter::with_capacity(CAPACITY, tmp);

        self.crypto_algorithm.decrypt(&mut reader, &mut writer)?;
        println!("decrypted");

        let mut tmp = writer.into_inner()?;
        tmp.rewind()?;

        let mut reader = BufReader::with_capacity(CAPACITY, tmp);

        self.pack_algorithm
            .decompression(&mut reader, &self.to_path)?;
        println!("unpacked");

        Ok(())
    }
}
