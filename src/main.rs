use std::{process, sync::Mutex};

use akasio::{run, Config, VERSION};
use anyhow::Result;
use clap::{Arg, Command};
use slog::{o, Drain};

fn parse() -> Result<Config> {
    // parse command line arguments
    let matches = Command::new("Akasio")
        .version(VERSION)
        .author("K4YT3X <i@k4yt3x.com>")
        .about("A simple Rust program that redirects HTTP requests")
        .arg(
            Arg::new("bind")
                .short('b')
                .long("bind")
                .value_name("BIND_ADDRESS")
                .help("binding IP address and port (IP:PORT)")
                .takes_value(true)
                .default_value("127.0.0.1:8000")
                .env("AKASIO_BIND_ADDRESS"),
        )
        .arg(
            Arg::new("redirect-table-path")
                .short('r')
                .long("redirect-table-path")
                .value_name("REDIRECT_TABLE_PATH")
                .help("path to the redirect table file")
                .takes_value(true)
                .default_value("akasio.json")
                .env("AKASIO_REDIRECT_TABLE_PATH"),
        )
        .get_matches();

    Ok(Config::new(
        {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
            slog::Logger::root(drain, o!())
        },
        matches.value_of_t_or_exit("bind"),
        matches.value_of_t_or_exit("redirect-table-path"),
    ))
}

#[tokio::main]
async fn main() {
    match parse() {
        Err(e) => {
            eprintln!("Program initialization error: {}", e);
            process::exit(1);
        }
        Ok(config) => process::exit(match run(config).await {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("Error: {}", e);
                1
            }
        }),
    }
}
