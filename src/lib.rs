use std::fs;

use actix_web::{get, web, HttpResponse, HttpServer};
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
    // read the redirect table and return the path's corresponding destination URL

    // read JSON file into a string
    let redirect_table = fs::read_to_string(&config.redirect_table_path);
    let redirect_table = match redirect_table {
        Ok(value) => value,
        Err(_e) => {
            error!(&config.logger, "Error reading the redirect table");
            return Err(Value::Null);
        }
    };

    // parse JSON string
    let value = serde_json::from_str(&redirect_table);
    let value = match value {
        Ok(value) => value,
        Err(_e) => {
            error!(&config.logger, "Error reading the redirect table");
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
        return Err(Value::Null);
    }

    // if everything works out fine, return target URL as a String
    Ok(target_url.to_string())
}

#[get("/{path}")]
async fn redirect(path: web::Path<String>, config: web::Data<Config>) -> HttpResponse {
    // read redirect destination into string
    let destination = match read_redirect_table(&config, path.to_string()) {
        Ok(destination) => destination,
        Err(_e) => {
            // if the read returned Null, return not found
            warn!(&config.logger, "Serving 404 for: /{}", path);
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not Found");
        }
    };

    // if the read succeeded, return 302 found
    info!(&config.logger, "Serving 302 for: /{}", path);
    return HttpResponse::Found()
        .append_header(("Location", destination))
        .finish();
}

pub async fn run(config: Config) -> Result<()> {
    let bind_address = config.bind.clone();
    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(config.clone()))
            .service(redirect)
    })
    .bind(bind_address)?
    .run()
    .await?;
    Ok(())
}
