#![feature(proc_macro_hygiene, decl_macro)]
// #[macro_use]
// extern crate diesel;
// extern crate dotenv;
#[macro_use]
extern crate rocket;

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
