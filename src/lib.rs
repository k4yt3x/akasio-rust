use std::fs;

use actix_web::{web, web::Data, HttpRequest, HttpResponse, HttpServer};
use anyhow::{anyhow, Result};
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

pub fn read_redirect_table(config: &Config, path: String) -> Result<Option<String>> {
    // read JSON file into a string
    let redirect_table = fs::read_to_string(&config.redirect_table_path)?;

    // parse JSON string
    let value: Value = serde_json::from_str(&redirect_table)?;

    // get target URL
    if let Some(target_url) = value.get(&path) {
        if target_url.is_string() {
            if let Some(target_string) = target_url.as_str() {
                return Ok(Some(target_string.to_string()));
            }
            else {
                return Err(anyhow!("failed to convert JSON value into string"));
            }
        }
    }

    Ok(None)
}

async fn redirect(request: HttpRequest, config: Data<Config>) -> HttpResponse {
    // read redirect destination into string
    // if the read succeeded, return 302 found
    match read_redirect_table(&config, request.path().to_string()) {
        Ok(lookup_result) => {
            if let Some(destination) = lookup_result {
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
            else {
                warn!(&config.logger, "404 not found {}", request.path());
                HttpResponse::NotFound()
                    .content_type("text/plain")
                    .body("NotFound")
            }
        }
        Err(error) => {
            error!(
                &config.logger,
                "500 internal server error {} {}",
                request.path(),
                error
            );
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
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
