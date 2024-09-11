pub type Addr2LineError = Box<dyn std::error::Error>;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("addr2line error")]
    Addr2Line(#[from] Addr2LineError),
    #[error("std io error")]
    IO(#[from] std::io::Error),
    #[error("Failed to read object file")]
    ObjectRead(#[from] object::read::Error),
    #[error("No debug symbols found")]
    NoDebugSymbols,
    #[error("Gimli error")]
    Gimli(#[from] addr2line::gimli::Error),
    #[error("Walkdir error")]
    Walkdir(#[from] walkdir::Error),
    #[error("Path to the project files must be absolute")]
    NotAbsolutePath,
    #[error("No main function symbol found in the binary")]
    NoMainFunction,
    #[error("No location in source found for address")]
    NoLocationFound,
    #[error("Errno was returned")]
    Errno(#[from] nix::errno::Errno),
}
