use anyhow::{Ok, Result};
use datadog_logs::{client::HttpDataDogClient, config::DataDogConfig, logger::DataDogLogger};
use log::*;
use std::env;

pub async fn initialize() -> Result<()> {
    let is_production = env::var("PRODUCTION").is_ok();

    if !is_production {
        return Ok(());
    }

    let mut config = DataDogConfig::default();

    config.apikey = env::var("DATADOG_API_KEY")?;
    config.service = Some(env::var("DATADOG_SERVICE_NAME")?);
    config.source = env::var("DATADOG_SERVICE_NAME")?;

    let client = HttpDataDogClient::new(&config).unwrap();
    let future = DataDogLogger::set_nonblocking_logger(client, config, LevelFilter::Info).unwrap();

    rocket::tokio::spawn(future);
    Ok(())
}
