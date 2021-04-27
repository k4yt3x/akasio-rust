/*
Name: Akasio (Rust)
Creator: K4YT3X
Date Created: April 27, 2021
Last Modified: April 27, 2021

Licensed under the GNU General Public License Version 3 (GNU GPL v3),
    available at: https://www.gnu.org/licenses/gpl-3.0.txt
(C) 2021 K4YT3X
*/

use actix_web::{get, web, HttpResponse, HttpServer};
use clap::Arg;
use log::{error, info, warn};
use serde_json::Value;
use std::fs;
use std::result;

fn read_redirect_table(path: String) -> result::Result<String, Value> {
    // read the redirect table and return the path's corresponding destination URL

    // read JSON file into a string
    let redirect_table = fs::read_to_string("akasio.json");
    let redirect_table = match redirect_table {
        Ok(value) => value,
        Err(_e) => {
            error!("Error reading the redirect table");
            return Err(Value::Null);
        }
    };

    // parse JSON string
    let value = serde_json::from_str(&redirect_table);
    let value = match value {
        Ok(value) => value,
        Err(_e) => {
            error!("Error reading the redirect table");
            Value::Null
        }
    };

    // create lookup string
    let mut lookup_string = "/".to_owned();
    lookup_string.push_str(&path);

    // get target URL
    let target_url = &value[&lookup_string];

    // if key is not found, return Null
    if target_url == &Value::Null {
        warn!("Path {} was not found in the redirect table", lookup_string);
        return Err(Value::Null);
    }

    // if everything works out fine, return target URL as a String
    Ok(target_url.to_string())
}

#[get("/{path}")]
async fn redirect(path: web::Path<String>) -> HttpResponse {
    // HTTP request handler

    // read redirect destination into string
    let destination = match read_redirect_table(path.to_string()) {
        Ok(destination) => destination,
        Err(_e) => {
            // if the read returned Null, return not found
            info!("Path not found, serving 404");
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not Found");
        }
    };

    // if the read succeeded, return 302 found
    info!("Path found, serving 302");
    return HttpResponse::Found()
        .header("Location", destination)
        .finish();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // program entry point

    // parse command line arguments
    let matches = clap::App::new("Akasio Redirector")
        .version("0.1.0")
        .author("K4YT3X <k4yt3x@k4yt3x.com>")
        .about("A simple Rust program that redirects HTTP requests.")
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("BIND_ADDRESS")
                .help("binding IP address and port (IP:PORT)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("redirect-table-path")
                .short("r")
                .long("redirect-table-path")
                .value_name("REDIRECT_TABLE_PATH")
                .help("path to the redirect table file")
                .takes_value(true),
        )
        .get_matches();

    // read matches into variables
    let bind = matches.value_of("bind").unwrap_or("127.0.0.1:8000");
    let _redirect_table_path = matches
        .value_of("redirect-table-path")
        .unwrap_or("akasio.json");

    // initialize logging
    env_logger::init();
    info!("Starting Akasio (Rust) server");

    // start server
    HttpServer::new(|| actix_web::App::new().service(redirect))
        .bind(bind)?
        .run()
        .await
}
