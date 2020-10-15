use rocket::tokio::io::{AsyncRead, AsyncWrite};
use serde::Serialize;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::marker::Unpin;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Error of the Cmd Execution.
#[derive(Debug)]
pub struct CmdError {
    pub code: i32,
    pub reason: String,
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.code, self.reason)
    }
}

impl Error for CmdError {}

impl CmdError {
    pub fn new<T: Into<String>>(code: i32, reason: T) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }
}

/// Result of the Cmd Execution.
pub type CmdResult = Result<(), CmdError>;

/// Options of the Index repository.
#[derive(Debug)]
pub struct IndexOptions {
    /// The location of the Index repository.
    pub path: String,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexOptions {
    /// Create a new Options for Index repository.
    pub fn new() -> Self {
        Self {
            path: "crates.io-index".to_string(),
        }
    }
}

/// Data for Generic Response.
#[derive(Serialize)]
pub struct ResponseData<'a, T> {
    code: usize,
    #[serde(rename = "type")]
    type_: &'a str,
    message: String,
    data: T,
}

impl<'a, T> ResponseData<'a, T>
where
    T: Serialize,
{
    /// Returns a new Generic Response.
    pub fn new(code: usize, message: String, data: T) -> Self {
        Self {
            code,
            type_: "unknown",
            message,
            data,
        }
    }

    /// Return a new Success Generic Response.
    pub fn success(data: T) -> Self {
        Self::new(200, "".into(), data)
    }
}

/// Read from multiple Readable.
pub struct Readers {
    readers: Vec<Box<dyn Read>>,
}

unsafe impl Send for Readers {}
unsafe impl Sync for Readers {}

impl Read for Readers {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut offset: usize = 0;
        for r in self.readers.iter_mut() {
            offset += r.read(&mut buf[offset..])?;
        }
        Ok(offset)
    }
}

impl AsyncRead for Readers {
    unsafe fn prepare_uninitialized_buffer(&self, _buf: &mut [MaybeUninit<u8>]) -> bool {
        false
    }

    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Read::read(&mut self.get_mut(), buf))
    }
}

impl Readers {
    /// Create a new Readers.
    pub fn new(readers: Vec<Box<dyn Read>>) -> Self {
        Self { readers }
    }
}

/// AsyncRead Wrapper.
pub struct AsyncReader<T: Read + Send>(pub T);

impl<T> Deref for AsyncReader<T>
where
    T: Read + Send,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AsyncReader<T>
where
    T: Read + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Read for AsyncReader<T>
where
    T: Read + Send,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<T> AsyncRead for AsyncReader<T>
where
    T: Read + Send + Unpin,
{
    unsafe fn prepare_uninitialized_buffer(&self, _buf: &mut [MaybeUninit<u8>]) -> bool {
        false
    }

    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Read::read(&mut self.get_mut().0, buf))
    }
}

/// AsyncWrite Wrapper.
pub struct AsyncWriter<T: Write + Send>(pub T);

impl<T> AsyncWrite for AsyncWriter<T>
where
    T: Write + Send + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Write::write(&mut self.get_mut().0, buf))
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(io::Write::flush(&mut self.0))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}
