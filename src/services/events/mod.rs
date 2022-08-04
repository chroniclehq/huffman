use crate::drivers::SQS;
use crate::services;

use anyhow::{anyhow, Result};
use aws_sdk_sqs::Client;
use rocket::tokio::{select, task, time};
use rocket::Shutdown;
use std::env;

pub struct EventChannel {
    _client: Client,
    _queue: String,
}

async fn handler(message: String) -> Result<()> {
    let owned_message = message.clone();
    let process = task::spawn_blocking(|| async move {
        let storage = services::storage::initialize().await?;
        let result = services::image::generate(owned_message.as_str(), &storage).await;
        result
    });

    if let Ok(handle) = process.await {
        let res = handle.await;
        match res {
            Ok(_) => {
                println!("Created variant for {}", message);
                Ok(())
            }
            Err(error) => {
                println!("{:?}", error);
                Err(anyhow!("Could not process {}", message))
            }
        }
    } else {
        Err(anyhow!("Could not spawn task for {}", message))
    }
}

impl EventChannel {
    pub async fn send_message(&self, message: &str) -> Result<()> {
        let result = SQS::send_message(&self._client, &self._queue, message).await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                println!("{:?}", error);
                Err(anyhow!("Could not send message"))
            }
        }
    }

    pub async fn listen(&self, mut shutdown: Shutdown) {
        let poll_interval = env::var("SQS_POLL_INTERVAL")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        loop {
            select! {
                _ = time::sleep(time::Duration::from_secs(poll_interval)) =>
                {
                    println!("Polling for messages");
                    let _ = SQS::receive_messages(&self._client, &self._queue, handler).await;
                    ()
                },
                _ = &mut shutdown => {
                    println!("Shutting down consumer");
                    break;
                },
            }
        }
    }
}

pub async fn initialize() -> Result<EventChannel> {
    let _client = SQS::create_client().await;
    let _queue = env::var("SQS_URL")?;

    Ok(EventChannel { _client, _queue })
}
