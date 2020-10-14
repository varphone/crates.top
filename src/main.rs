#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use crates_top::route;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/crates/index", route::crates::index::routes())
        .launch();
}
