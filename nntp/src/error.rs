use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection closed normally")]
    ConnectionClosed,
    #[error("Trying to work with closed connection")]
    AlreadyClosed,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("TLS error: {0}")]
    Tls(#[from] TlsError),
    #[error("NNTP error: {0}")]
    Nntp(#[from] NntpError),
}

#[allow(missing_copy_implementations)]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TlsError {
    /// Native TLS error.
    #[cfg(feature = "native-tls")]
    #[error("native-tls error: {0}")]
    Native(#[from] native_tls_crate::Error),
    /// Rustls error.
    #[cfg(feature = "__rustls-tls")]
    #[error("rustls error: {0}")]
    Rustls(#[from] rustls::Error),
    /// DNS name resolution error.
    #[cfg(feature = "__rustls-tls")]
    #[error("Invalid DNS name")]
    InvalidDnsName,
    #[error("Tls Not Enable")]
    NotEnabled,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum NntpError {
    #[error("invalid status code")]
    InvalidStatusCode,
}
