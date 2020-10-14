#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use crates_top::{route, IndexOptions};
use rocket::fairing::AdHoc;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/crates/index", route::crates::index::routes())
        .attach(AdHoc::on_attach("IndexDir Config", |rocket| {
            // println!("config={:?}", rocket.config());
            let mut index_options = IndexOptions::default();
            if let Ok(table) = rocket.config().get_table("index_options") {
                if let Some(s) = table.get("path") {
                    index_options.path = s.as_str().unwrap().to_string();
                }
            }

            Ok(rocket.manage(index_options))
        }))
        .launch();
}
