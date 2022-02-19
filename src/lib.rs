use std::{fs, result};

use actix_web::{get, web, HttpResponse, HttpServer};
use anyhow::Result;
use serde_json::Value;
use slog::{error, info, warn};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Config
{
    logger: slog::Logger,
    bind: String,
    redirect_table_path: String,
}

impl Config
{
    pub fn new(logger: slog::Logger, bind: String, redirect_table_path: String) -> Config
    {
        Config {
            logger,
            bind,
            redirect_table_path,
        }
    }
}

pub fn read_redirect_table(config: &Config, path: String) -> result::Result<String, Value>
{
    // read the redirect table and return the path's corresponding destination URL

    // read JSON file into a string
    let redirect_table = fs::read_to_string("akasio.json");
    let redirect_table = match redirect_table {
        Ok(value) => value,
        Err(_e) => {
            error!(config.logger, "Error reading the redirect table");
            return Err(Value::Null);
        }
    };

    // parse JSON string
    let value = serde_json::from_str(&redirect_table);
    let value = match value {
        Ok(value) => value,
        Err(_e) => {
            error!(config.logger, "Error reading the redirect table");
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
        warn!(
            config.logger,
            "Path {} was not found in the redirect table", lookup_string
        );
        return Err(Value::Null);
    }

    // if everything works out fine, return target URL as a String
    Ok(target_url.to_string())
}

#[get("/{path}")]
async fn redirect(config: &Config, path: web::Path<String>) -> HttpResponse
{
    // HTTP request handler

    // read redirect destination into string
    let destination = match read_redirect_table(&config, path.to_string()) {
        Ok(destination) => destination,
        Err(_e) => {
            // if the read returned Null, return not found
            info!(config.logger, "Path not found, serving 404");
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not Found");
        }
    };

    // if the read succeeded, return 302 found
    info!(config.logger, "Path found, serving 302");
    return HttpResponse::Found()
        .header("Location", destination)
        .finish();
}

async fn run(bind: &str) -> Result<()>
{
    HttpServer::new(|| actix_web::App::new().service(redirect))
        .bind(bind)?
        .run()
        .await?;
    Ok(())
}