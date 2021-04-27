/*
Name: Akasio (Rust)
Creator: K4YT3X
Date Created: April 27, 2021
Last Modified: April 27, 2021

Licensed under the GNU General Public License Version 3 (GNU GPL v3),
    available at: https://www.gnu.org/licenses/gpl-3.0.txt
(C) 2021 K4YT3X
*/

use actix_web::{get, web, App as ActixApp, HttpResponse, HttpServer};
use clap::{App, Arg};
use serde_json::Value;
use std::fs;
use std::result;

fn read_redirect_table(path: String) -> result::Result<String, Value> {
    // read the redirect table and return the path's corresponding destination URL

    // read JSON file into a string
    let redirect_table = fs::read_to_string("akasio.json").unwrap();

    // parse JSON string
    let value = serde_json::from_str(&redirect_table);

    // if the read operation failed, let value be Null
    let value = match value {
        Ok(value) => value,
        Err(_e) => Value::Null,
    };

    // create lookup string
    let mut lookup_string = "/".to_owned();
    lookup_string.push_str(&path);

    // get target URL
    let target_url = &value[&lookup_string];

    // if key is not found, return Null
    if target_url == &Value::Null {
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
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not Found");
        }
    };

    // if the read succeeded, return 302 found
    return HttpResponse::Found()
        .header("Location", destination)
        .finish();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // program entry point

    // parse command line arguments
    let matches = App::new("Akasio Redirector")
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
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("enable debugging mode"),
        )
        .get_matches();

    // read matches into variables
    let bind = matches.value_of("bind").unwrap_or("127.0.0.1:8000");
    // let debug = matches.is_present("debug");

    // start server
    HttpServer::new(|| ActixApp::new().service(redirect))
        .bind(bind)?
        .run()
        .await
}
