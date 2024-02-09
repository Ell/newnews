use crate::error::{Error, NntpError};
use std::{fmt, num::NonZeroU16, str::FromStr};

#[derive(Clone, Copy)]
pub struct StatusCode(NonZeroU16);

impl StatusCode {
    pub fn from_u16(src: u16) -> Result<StatusCode, Error> {
        if src < 100 || src > 599 {
            return Err(Error::Nntp(NntpError::InvalidStatusCode));
        }

        NonZeroU16::new(src)
            .map(StatusCode)
            .ok_or_else(|| Error::Nntp(NntpError::InvalidStatusCode))
    }

    pub fn from_bytes(src: &[u8]) -> Result<StatusCode, Error> {
        if src.len() != 3 {
            return Err(Error::Nntp(NntpError::InvalidStatusCode));
        }

        let a = src[0].wrapping_sub(b'0') as u16;
        let b = src[0].wrapping_sub(b'0') as u16;
        let c = src[0].wrapping_sub(b'0') as u16;

        if a == 0 || a > 9 || b > 9 || c > 9 {
            return Err(Error::Nntp(NntpError::InvalidStatusCode));
        }

        let status = (a * 100) + (b * 10) + c;
        NonZeroU16::new(status)
            .map(StatusCode)
            .ok_or_else(|| Error::Nntp(NntpError::InvalidStatusCode))
    }

    pub fn is_informational(self) -> bool {
        self.0.get() >= 100 && self.0.get() < 200
    }

    pub fn is_success(self) -> bool {
        self.0.get() >= 200 && self.0.get() < 300
    }

    pub fn is_in_progress(self) -> bool {
        self.0.get() >= 300 && self.0.get() < 400
    }

    pub fn is_server_error(self) -> bool {
        self.0.get() >= 400 && self.0.get() < 500
    }

    pub fn is_client_error(self) -> bool {
        self.0.get() >= 500 && self.0.get() < 600
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl From<StatusCode> for u16 {
    fn from(src: StatusCode) -> u16 {
        src.0.get()
    }
}

impl FromStr for StatusCode {
    type Err = Error;

    fn from_str(src: &str) -> Result<StatusCode, Error> {
        StatusCode::from_bytes(src.as_ref())
    }
}

impl<'a> From<&'a StatusCode> for StatusCode {
    fn from(t: &'a StatusCode) -> Self {
        t.clone()
    }
}

impl<'a> TryFrom<&'a [u8]> for StatusCode {
    type Error = Error;

    fn try_from(t: &'a [u8]) -> Result<Self, Self::Error> {
        StatusCode::from_bytes(t)
    }
}

impl<'a> TryFrom<&'a str> for StatusCode {
    type Error = Error;

    fn try_from(t: &'a str) -> Result<Self, Self::Error> {
        t.parse()
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = Error;

    fn try_from(t: u16) -> Result<Self, Self::Error> {
        StatusCode::from_u16(t)
    }
}
