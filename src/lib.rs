#![feature(proc_macro_hygiene, decl_macro)]
// #[macro_use]
// extern crate diesel;
// extern crate dotenv;
#[macro_use]
extern crate rocket;

use serde::Serialize;

pub mod route;

#[derive(Debug)]
pub struct IndexOptions {
    pub path: String,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexOptions {
    pub fn new() -> Self {
        Self {
            path: "crates.io-index".to_string(),
        }
    }
}

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
    fn new(code: usize, message: String, data: T) -> Self {
        Self {
            code,
            type_: "unknown",
            message,
            data,
        }
    }

    pub fn success(data: T) -> Self {
        Self::new(200, "".into(), data)
    }
}
