use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid procotol")]
    InvalidProtocol,
    #[error("internal error")]
    InternalErr,
    #[error("io error: {error}")]
    IoErr {
        #[from]
        #[source]
        error: ::std::io::Error,
    },
}
