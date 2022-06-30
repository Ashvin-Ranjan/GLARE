//! Starts the server based on arguments passed in
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

use clap::Parser;
use colored::Colorize;
use std::fs;
use utils::FormatUnpack;

mod data;
mod utils;
mod version;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Warn user about new versions if they do not choose to disable it
    if !args.no_warn {
        if let Some(v) = version::check_new_version() {
            println!(
                "{}: The current version of this program is v{} while the newest version is v{}",
                "Warning".yellow(),
                env!("CARGO_PKG_VERSION").blue(),
                v.blue()
            );
        }
    }

    // Check for token file
    let possible_token = match fs::read_to_string(".token") {
        Ok(data) => Some(data),
        Err(_) => None,
    };

    data::aggregate_data(args.author, args.name, args.teams, possible_token)
        .await
        .fup();

    return Ok(());
}
