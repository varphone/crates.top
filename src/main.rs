use clap::{App, Arg, SubCommand};
use crates_top::cmd;

fn main() {
    let matches = App::new("Crates.top, the top overlay of crates.io")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Varphone Wong <varphone@qq.com>")
        .about("https://github.com/varphone/crates.top")
        .subcommand(
            SubCommand::with_name("admin")
                .about("Administrative Utilities")
                .arg(Arg::with_name("dummy").required(false)),
        )
        .subcommand(
            SubCommand::with_name("web")
                .about("Start Web Service (Foreground)")
                .arg(Arg::with_name("dummy").required(false)),
        )
        .get_matches();

    let r = match matches.subcommand() {
        ("admin", Some(matches)) => cmd::admin::run(matches),
        ("web", Some(matches)) => cmd::web::run(matches),
        _ => Ok(()),
    };

    std::process::exit(r.map_or_else(|err| err.code, |_| 0));
}
