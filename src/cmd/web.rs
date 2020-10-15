use crate::{route, CmdError, CmdResult, IndexOptions};

use clap::ArgMatches;
use rocket::fairing::AdHoc;

pub async fn run(_args: &ArgMatches<'_>) -> CmdResult {
    rocket::ignite()
        .attach(AdHoc::on_launch("Config Printer", |rocket| {
            println!("Rocket launch config: {:#?}", rocket.config());
        }))
        .attach(AdHoc::on_attach("IndexDir Config", |mut rocket| async {
            // println!("config={:?}", rocket.config());
            let mut index_options = IndexOptions::default();
            if let Ok(table) = rocket.config().await.get_table("index_options") {
                if let Some(s) = table.get("path") {
                    index_options.path = s.as_str().unwrap().to_string();
                }
            }

            Ok(rocket.manage(index_options))
        }))
        // .mount("/", routes![index])
        .mount("/api/v1/crates", route::api::v1::crates::routes())
        .mount("/crates/index", route::crates::index::routes())
        .launch()
        .await
        .map_err(|err| CmdError::new(0, format!("{}", err)))
    // Ok(())
}
