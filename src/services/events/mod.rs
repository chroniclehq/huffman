pub mod message;

use crate::drivers::SQS;
use crate::services;

use self::message::Message;
use anyhow::{anyhow, Result};
use aws_sdk_sqs::Client;
use rocket::tokio::{select, task, time};
use rocket::Shutdown;
use std::env;

pub struct EventChannel {
    _client: Client,
    _queue: String,
}

async fn handler(data: String) -> Result<()> {
    if let Some(message) = message::deserialize(data.as_str()) {
        // Clone the message and spawn a blocking task to generate the variants.
        // Cloning is necessary since value will be moved to make it thread safe.
        let owned_message = message.clone();
        let process = task::spawn_blocking(|| async move {
            let storage = services::storage::initialize().await?;
            let result = services::image::generate(&owned_message.url, &storage).await;
            result
        });

        // Tokio's task::spawn_blocking returns a task handle which must be awaited to start task
        // which in turn returns a promise to the inner async scope.
        if let Ok(handle) = process.await {
            let res = handle.await;
            match res {
                Ok(_) => {
                    println!("Created variant for {}", &message.url);
                    Ok(())
                }
                Err(error) => {
                    println!("{:?}", error);
                    Err(anyhow!("Could not process {}", &message.url))
                }
            }
        } else {
            Err(anyhow!("Could not spawn task for {}", &message.url))
        }
    } else {
        Err(anyhow!("Could not deserialize message: {}", data))
    }
}

impl EventChannel {
    pub async fn send_message(&self, message: &Message) -> Result<()> {
        if let Some(data) = message::serialize(message) {
            let result = SQS::send_message(&self._client, &self._queue, &data).await;

            match result {
                Ok(_) => Ok(()),
                Err(error) => {
                    println!("{:?}", error);
                    Err(anyhow!("Could not send message"))
                }
            }
        } else {
            Err(anyhow!("Could not serialize message"))
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
