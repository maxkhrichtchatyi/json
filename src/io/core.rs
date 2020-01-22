//! Reimplements core logic and types from `std::io` in an `alloc`-friendly
//! fashion.

use lib::*;

pub enum ErrorKind {
    Other,
}

pub struct Error(&'static str);

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.0, formatter)
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, error: &'static str) -> Error {
        let _ = kind;
        Error(error)
    }
}

pub type Result<T> = result::Result<T, Error>;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        // All our Write impls in no_std mode always write the whole buffer in
        // one call infallibly.
        let result = self.write(buf);
        debug_assert!(result.is_ok());
        debug_assert_eq!(result.unwrap_or(0), buf.len());
        Ok(())
    }

    fn flush(&mut self) -> Result<()>;
}

impl<W: Write> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (*self).write(buf)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (*self).write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        (*self).flush()
    }
}

impl Write for Vec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Bytes { inner: self }
    }
}

pub struct Bytes<R> {
    inner: R,
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Result<u8>> {
        let mut byte = 0;
        match self.inner.read(slice::from_mut(&mut byte)) {
            Ok(0) => None,
            Ok(..) => Some(Ok(byte)),
            Err(e) => Some(Err(e)),
        }
    }
}