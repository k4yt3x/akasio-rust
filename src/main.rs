use std::{process, sync::Mutex};

use akasio::{run, Config};
use anyhow::Result;
use clap::Parser;
use slog::{o, Drain};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // binding IP address and port (IP:PORT)
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    bind: String,

    // path to the redirect table file
    #[arg(short, long, default_value = "/etc/akasio.json")]
    table: String,
}

fn parse() -> Result<Config> {
    let args = Args::parse();

    Ok(Config::new(
        {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
            slog::Logger::root(drain, o!())
        },
        args.bind,
        args.table,
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
