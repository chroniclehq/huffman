use anyhow::{Ok, Result};
use datadog_logs::{client::HttpDataDogClient, config::DataDogConfig, logger::DataDogLogger};
use log::*;
use std::env;

pub async fn initialize() -> Result<()> {
    let enable_data_dog = env::var("DATADOG_API_KEY").is_ok();

    if !enable_data_dog {
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
