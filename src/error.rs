#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorKind {
    EncryptionError,
    DecryptionError,
    InvalidKeyfile,
}

impl ErrorKind {
    pub fn description(&self) -> &str {
        match self {
            ErrorKind::EncryptionError => "encrypt failed",
            ErrorKind::DecryptionError => "decrypt failed",
            ErrorKind::InvalidKeyfile => "invalid keyfile",
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind.description())
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl std::error::Error for Error {}
