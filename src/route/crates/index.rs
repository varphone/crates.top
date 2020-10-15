use crate::{AsyncReader, AsyncWriter, IndexOptions, Readers, ResponseData};

use rocket::data::{Data, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::response::{Debug, Response};
use rocket::{get, post, routes};
use rocket::{Route, State};
use rocket_contrib::json::Json;
use std::io::{self, Cursor, Read, Write};
use std::process::{Command, Stdio};
use std::string::String;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/info/refs?service=git-upload-pack")]
fn fetch_refs(index_options: State<IndexOptions>) -> Result<Response<'_>, Debug<io::Error>> {
    let output = Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&index_options.path)
        .output()?;

    let mut buffer: Vec<u8> = vec![];
    buffer.extend_from_slice(b"001e# service=git-upload-pack\n0000");
    buffer.extend_from_slice(&output.stdout);

    let response = Response::build()
        .status(Status::Ok)
        .raw_header("Cache-Control", "no-cache, max-age=0, must-revalidate")
        .header(ContentType::new(
            "application",
            "x-git-upload-pack-advertisement",
        ))
        .raw_header("Expires", "Fri, 01 Jan 1980 00:00:00 GMT")
        .raw_header("Pragma", "no-cache")
        .raw_header("Set-Cookie", "lang=en-US; Path=/; Max-Age=2147483647")
        .sized_body(buffer.len(), Cursor::new(buffer))
        // .streamed_body(Readers::new(vec![
        //     Box::new(Cursor::new(b"001e# service=git-upload-pack\n0000")),
        //     Box::new(Cursor::new(output.stdout)),
        // ]))
        .ok();

    response
    // Ok(response)
}

#[post(
    "/git-upload-pack",
    // format = "x-git-upload-pack-request",
    data = "<data>"
)]
async fn fetch_content(
    data: Data,
    index_options: State<'_, IndexOptions>,
) -> Result<Response<'_>, Debug<io::Error>> {
    let mut process = Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")
        .arg(&index_options.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = process
        .stdin
        .as_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "Standard Input"))?;
    data.open(4usize.kibibytes())
        .stream_to(AsyncWriter(stdin))
        .await?;

    let stdout = process
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "Standard Output"))?;

    let response = Response::build()
        .status(Status::Ok)
        .header(ContentType::new("application", "x-git-upload-pack-result"))
        .streamed_body(AsyncReader(stdout))
        .ok();

    response
}

const SYNC_CMD: &str = "git fetch upstream && git checkout -b tmp master && git rebase upstream/master && git checkout master && git branch -D tmp && git rebase upstream/master";

#[get("/sync")]
fn sync(index_options: State<IndexOptions>) -> Json<ResponseData<String>> {
    // Fetch and test rebase
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir(&index_options.path)
            .args(&["/C", SYNC_CMD])
            .output()
    } else {
        Command::new("sh")
            .current_dir(&index_options.path)
            .arg("-c")
            .arg(SYNC_CMD)
            .output()
    };

    match output {
        Ok(output) => {
            io::stdout().write_all(&output.stderr).unwrap();
            if output.status.success() {
                Json(ResponseData::success("".into()))
            } else {
                Json(ResponseData::new(
                    500,
                    "Repo rebase failure!".into(),
                    String::from_utf8(output.stderr).unwrap(),
                ))
            }
        }
        Err(err) => Json(ResponseData::new(500, err.to_string(), "".into())),
    }
}

pub fn routes() -> Vec<Route> {
    routes![index, fetch_refs, fetch_content, sync]
}
