use serde::Serialize;
use std::error::Error;
use std::fmt;

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
