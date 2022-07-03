//! Starts the server based on arguments passed in
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

use clap::Parser;
use colored::Colorize;
use std::{fs, sync::Mutex};
use utils::err::FormatUnpack;

mod data;
mod filters;
mod handlers;
mod utils;
mod version;

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref POSSIBLE_TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Author of the repo
    #[clap(short, long, value_parser)]
    author: String,

    /// Name of the repo
    #[clap(short, long, value_parser)]
    name: String,

    /// Checks if the program should export to export.json
    #[clap(long, value_parser, default_value_t = false)]
    export: bool,

    /// Check teams instead of users
    #[clap(long, value_parser, default_value_t = false)]
    teams: bool,

    /// Which port the website will show up on
    #[clap(long, value_parser, default_value_t = 8080)]
    port: u16,

    /// Do not warn about newer versions
    #[clap(long, value_parser, default_value_t = false)]
    no_warn: bool,

    /// Do not check diffs of the PRs
    #[clap(long, value_parser, default_value_t = false)]
    no_diff: bool,

    /// Do not pretty print errors
    #[clap(long, value_parser, default_value_t = false)]
    no_pretty_errors: bool,
}

pub async fn reload_data() {
    if let Ok(mut d) = handlers::DATA.lock() {
        let mut p = None;
        if let Ok(t) = POSSIBLE_TOKEN.lock() {
            match &*t {
                Some(s) => p = Some(s.to_owned()),
                None => p = None,
            }
        }
        let pr_data = data::aggregate_data(
            ARGS.author.to_owned(),
            ARGS.name.to_owned(),
            ARGS.teams,
            ARGS.no_diff,
            ARGS.export,
            p,
        )
        .await
        .fup(ARGS.no_pretty_errors);
        *d = pr_data
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Warn user about new versions if they do not choose to disable it

    if !ARGS.no_warn {
        if let Some(v) = version::check_new_version() {
            println!(
                "{}: The current version of this program is v{} while the newest version is v{}",
                "Warning".yellow(),
                env!("CARGO_PKG_VERSION").blue(),
                v.blue()
            );
        }
    }

    if let Ok(mut t) = POSSIBLE_TOKEN.lock() {
        *t = match fs::read_to_string(".token") {
            Ok(data) => Some(data),
            Err(_) => None,
        };
        let pr_data = data::aggregate_data(
            ARGS.author.to_owned(),
            ARGS.name.to_owned(),
            ARGS.teams,
            ARGS.no_diff,
            ARGS.export,
            match &*t {
                Some(x) => Some(x.to_owned()),
                None => None,
            },
        )
        .await
        .fup(ARGS.no_pretty_errors);

        match handlers::DATA.lock() {
            Ok(mut d) => *d = pr_data,
            Err(_) => Err(format!(
                "`{}` variable is locked when it should not be.",
                "DATA".blue()
            ))
            .fup(ARGS.no_pretty_errors),
        }
    } else {
        Err(format!(
            "`{}` variable is locked when it should not be.",
            "POSSIBLE_TOKEN".blue()
        ))
        .fup(ARGS.no_pretty_errors)
    }

    let routes = filters::routes();

    warp::serve(routes).run(([0, 0, 0, 0], ARGS.port)).await;
    return Ok(());
}
