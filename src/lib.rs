use std::fs;

use actix_web::{web, web::Data, HttpRequest, HttpResponse, HttpServer};
use anyhow::Result;
use serde_json::Value;
use slog::{error, info, warn};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct Config {
    logger: slog::Logger,
    bind: String,
    redirect_table_path: String,
}

impl Config {
    pub fn new(logger: slog::Logger, bind: String, redirect_table_path: String) -> Config {
        Config {
            logger,
            bind,
            redirect_table_path,
        }
    }
}

pub fn read_redirect_table(config: &Config, path: String) -> Result<String, Value> {
    // read JSON file into a string
    let redirect_table = match fs::read_to_string(&config.redirect_table_path) {
        Ok(value) => value,
        Err(_e) => {
            error!(&config.logger, "Error reading the redirect table");
            return Err(Value::Null);
        }
    };

    // parse JSON string
    let value = match serde_json::from_str(&redirect_table) {
        Ok(value) => value,
        Err(_e) => {
            error!(&config.logger, "Error reading the redirect table");
            Value::Null
        }
    };

    // get target URL
    let target_url = &value[path];

    // if key is not found, return Null
    if target_url == &Value::Null {
        return Err(Value::Null);
    }

    // if everything works out fine, return target URL as a String
    Ok(target_url.to_string())
}

async fn redirect(request: HttpRequest, config: Data<Config>) -> HttpResponse {
    // read redirect destination into string
    // if the read succeeded, return 302 found
    if let Ok(destination) = read_redirect_table(&config, request.path().to_string()) {
        info!(
            &config.logger,
            "302 redirecting {} to {}",
            request.path(),
            destination
        );
        HttpResponse::Found()
            .append_header(("Location", destination))
            .finish()
    }
    // if the read returned Null, return 404 not found
    else {
        warn!(&config.logger, "404 not found for {}", request.path());
        HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not Found")
    }
}

pub async fn run(config: Config) -> Result<()> {
    info!(&config.logger, "Starting Akasio server {}", VERSION);
    let bind_address = config.bind.clone();
    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(Data::new(config.clone()))
            .default_service(web::route().to(redirect))
    })
    .bind(bind_address)?
    .run()
    .await?;
    Ok(())
}
