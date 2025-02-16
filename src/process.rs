use std::{
    cmp::max,
    fs::{self, File},
    io::{BufReader, BufWriter, Seek},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Result;
use tempfile::NamedTempFile;
use tokio::{
    fs::metadata,
    sync::mpsc::{channel, Receiver, Sender},
    time::sleep,
};

use crate::{Crypto, Pack};

const CAPACITY: usize = 8 * 1024 * 1024; // 8MiB

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Enc,
    Dec,
}

#[derive(Clone, Debug)]
struct ProgressMessage {
    // TODO: replace trait
    process_target: PathBuf,
    process_target_size: usize,
}

pub struct Process {
    target: Target,
    pack_algorithm: Box<dyn Pack>,
    crypto_algorithm: Box<dyn Crypto>,

    from_path: PathBuf,
    to_path: PathBuf,

    bypass_progress: Option<Sender<u8>>,
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
            bypass_progress: None,
        }
    }

    pub fn bypass_progress(self, tx: Sender<u8>) -> Self {
        Self {
            bypass_progress: Some(tx),
            ..self
        }
    }

    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.to_path.exists() {
            let e = std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("{:?} already exists", self.to_path),
            );
            return Err(Box::new(e));
        }
        match self.target {
            Target::Enc => {
                self.enc().await?;
            }
            Target::Dec => {
                self.dec().await?;
            }
        };
        Ok(())
    }

    async fn enc(self) -> Result<(), Box<dyn std::error::Error>> {
        let (mtx, mrx) = channel::<ProgressMessage>(4);
        let tx = self.bypass_progress.clone();
        if let Some(tx) = tx {
            self.spawn_progress(mrx, tx).await;
        };

        let r: anyhow::Result<()> = tokio::task::spawn_blocking(move || {
            let tmp = NamedTempFile::new()?;
            let dst = File::create(&self.to_path)?;

            if self.bypass_progress.is_some() {
                let _ = mtx.blocking_send(ProgressMessage {
                    process_target: tmp.path().to_path_buf(),
                    process_target_size: get_fs_size(&self.from_path).unwrap_or(1),
                });
            }

            let mut writer = BufWriter::with_capacity(CAPACITY, tmp);

            self.pack_algorithm
                .compression(self.from_path.as_path(), &mut writer)
                .unwrap();

            let mut tmp = writer.into_inner()?;
            tmp.rewind().unwrap();

            if self.bypass_progress.is_some() {
                let _ = mtx.blocking_send(ProgressMessage {
                    process_target: self.to_path.clone(),
                    process_target_size: get_fs_size(tmp.path()).unwrap_or(1),
                });
            }

            let mut reader = BufReader::with_capacity(CAPACITY, tmp);
            let mut writer = BufWriter::with_capacity(CAPACITY, dst);

            self.crypto_algorithm
                .encrypt(&mut reader, &mut writer)
                .unwrap();

            Ok(())
        })
        .await?;
        r?;
        Ok(())
    }

    async fn dec(self) -> Result<(), Box<dyn std::error::Error>> {
        let (mtx, mrx) = channel::<ProgressMessage>(4);
        let tx = self.bypass_progress.clone();
        if let Some(tx) = tx {
            self.spawn_progress(mrx, tx).await;
        };

        let r: anyhow::Result<()> = tokio::task::spawn_blocking(move || {
            let src = File::open(&self.from_path)?;
            let tmp = NamedTempFile::new()?;

            if self.bypass_progress.is_some() {
                let _ = mtx.blocking_send(ProgressMessage {
                    process_target: tmp.path().into(),
                    process_target_size: get_fs_size(&self.from_path).unwrap_or(1),
                });
            }

            let mut reader = BufReader::with_capacity(CAPACITY, src);
            let mut writer = BufWriter::with_capacity(CAPACITY, tmp);

            self.crypto_algorithm.decrypt(&mut reader, &mut writer)?;

            let mut tmp = writer.into_inner()?;
            tmp.rewind()?;

            if self.bypass_progress.is_some() {
                let _ = mtx.blocking_send(ProgressMessage {
                    process_target: self.to_path.clone(),
                    process_target_size: get_fs_size(tmp.path()).unwrap_or(1),
                });
            }

            let mut reader = BufReader::with_capacity(CAPACITY, tmp);

            self.pack_algorithm
                .decompression(&mut reader, &self.to_path)?;

            Ok(())
        })
        .await?;
        r?;
        Ok(())
    }

    async fn spawn_progress(&self, mut mrx: Receiver<ProgressMessage>, tx: Sender<u8>) {
        tokio::spawn(async move {
            let first_phase = mrx.recv().await.unwrap();
            loop {
                sleep(Duration::from_millis(100)).await;

                let meta = metadata(&first_phase.process_target).await.unwrap();
                let need = max(first_phase.process_target_size, 1) as f64 * 0.98;
                let curr = meta.len() as f64;

                if need <= curr {
                    break;
                }

                let progress = ((curr / need) * u8::MAX as f64).round() as u8;
                let progress = progress / 2;

                tx.send(progress).await.unwrap();
            }
            let second_phase = mrx.recv().await.unwrap();
            loop {
                sleep(Duration::from_millis(100)).await;

                let len = if second_phase.process_target.is_dir() {
                    get_fs_size(&second_phase.process_target).unwrap() as u64
                } else {
                    metadata(&second_phase.process_target).await.unwrap().len()
                };
                let need = max(second_phase.process_target_size, 1) as f64 * 0.98;
                let curr = len as f64;

                if need <= curr {
                    tx.send(u8::MAX).await.unwrap();
                    break;
                }

                let progress = ((curr / need) * u8::MAX as f64).round() as u8;
                let progress = u8::MAX / 2 + progress / 2;

                tx.send(progress.clamp(u8::MAX / 2, u8::MAX)).await.unwrap();
            }
        });
    }
}

fn get_fs_size(path: impl AsRef<Path>) -> Result<usize> {
    let path = path.as_ref();

    if path.is_file() {
        return Ok(path.metadata()?.len() as usize);
    }

    let mut total_size = 0;
    let mut stack = vec![path.to_path_buf()];

    while let Some(current_path) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&current_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    } else if metadata.is_dir() {
                        stack.push(entry.path());
                    }
                }
            }
        }
    }

    Ok(total_size as usize)
}

#[cfg(test)]
mod test {
    use std::{
        fs::{create_dir, File},
        io::Write,
    };

    use tempfile::{tempdir, NamedTempFile};

    use super::get_fs_size;

    #[test]
    fn get_fs_size_test() {
        // dir
        let td = tempdir().unwrap();
        for i in 0..3 {
            let mut f = File::create(td.path().join(format!("file{i}"))).unwrap();
            f.write_all("a".as_bytes()).unwrap();
        }
        create_dir(td.path().join("depth")).unwrap();
        for i in 0..3 {
            let mut f = File::create(td.path().join("depth").join(format!("file{i}"))).unwrap();
            f.write_all("a".as_bytes()).unwrap();
        }

        let size = get_fs_size(td.path()).unwrap();
        assert_eq!(6, size);

        let mut f = NamedTempFile::new().unwrap();
        f.write_all("7 bytes".as_bytes()).unwrap();
        assert_eq!(7, get_fs_size(f.path()).unwrap());
    }
}
