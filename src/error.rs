use std::{
    io::{self, ErrorKind},
    num::ParseIntError,
};

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[source] io::Error),
    #[error("The peer has disconnect")]
    Disconnect,
    #[error("Cannot parse the data")]
    Parse,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        if err.kind() == ErrorKind::UnexpectedEof {
            return Self::Disconnect;
        }

        Self::Io(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::Parse
    }
}
