use crate::{route, CmdResult, IndexOptions};

use clap::ArgMatches;
use rocket::fairing::AdHoc;

pub fn run(_args: &ArgMatches<'_>) -> CmdResult {
    rocket::ignite()
        // .mount("/", routes![index])
        .mount("/api/v1/crates", route::api::v1::crates::routes())
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
    Ok(())
}
