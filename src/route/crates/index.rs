use crate::IndexOptions;
use rocket::http::{ContentType, Status};
use rocket::request::Form;
use rocket::{Data, Response, Route, State};
use std::io::{Cursor, Read, Result as IoResult, Seek, SeekFrom};
use std::process::{Command, Stdio};

struct Readers {
    readers: Vec<Box<dyn Read>>,
}

impl Read for Readers {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let mut offset: usize = 0;
        for r in self.readers.iter_mut() {
            offset += r.read(&mut buf[offset..])?;
        }
        Ok(offset)
    }
}

impl Readers {
    pub fn new(readers: Vec<Box<dyn Read>>) -> Self {
        Self { readers }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/info/refs?service=git-upload-pack")]
fn fetch_refs(index_options: State<IndexOptions>) -> Option<Response<'static>> {
    let mut output = Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&index_options.path)
        .output()
        .expect("failed to execute process");

    let response = Response::build()
        .status(Status::Ok)
        .header(ContentType::new(
            "application",
            "x-git-upload-pack-advertisement",
        ))
        .raw_header("Cache-Control", "no-cache, max-age=0, must-revalidate")
        .raw_header("Expires", "Fri, 01 Jan 1980 00:00:00 GMT")
        .raw_header("Pragma", "no-cache")
        .streamed_body(Readers::new(vec![
            Box::new(Cursor::new(b"001e# service=git-upload-pack\n0000")),
            Box::new(Cursor::new(output.stdout)),
        ]))
        .finalize();

    Some(response)
}

#[post(
    "/git-upload-pack",
    // format = "x-git-upload-pack-request",
    data = "<data>"
)]
fn fetch_content(data: Data, index_options: State<IndexOptions>) -> Option<Response<'static>> {
    let mut process = match Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")
        .arg(&index_options.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn git: {}", why),
        Ok(process) => process,
    };

    let stdin = process.stdin.as_mut().expect("Failed to open stdin");
    data.stream_to(stdin).map(|n| format!("Wrote {} bytes.", n));

    let response = Response::build()
        .status(Status::Ok)
        .header(ContentType::new("application", "x-git-upload-pack-result"))
        .streamed_body(process.stdout.take().unwrap())
        .finalize();

    Some(response)
}

pub fn routes() -> Vec<Route> {
    routes![index, fetch_refs, fetch_content]
}
