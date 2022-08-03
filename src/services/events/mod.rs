use crate::drivers::kafka::{self, extract_payload};
use crate::services;

use anyhow::{anyhow, Result};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rocket::futures::TryStreamExt;
use rocket::tokio::{select, task, time};
use rocket::Shutdown;

pub struct EventChannel {
    _producer: FutureProducer,
}

impl EventChannel {
    pub async fn send(&self, message: &str) -> Result<()> {
        let topic = String::from("image-optimizer");
        let key = "huffman";

        let record = FutureRecord::to(&topic).key(key).payload(message);
        let response = self
            ._producer
            .send(record, Timeout::After(time::Duration::from_secs(0)));

        match response.await {
            Ok(delivery) => {
                println!("Sent: {:?}", delivery);
                Ok(())
            }
            Err((e, _)) => {
                println!("Error: {:?}", e);
                Err(anyhow!("Could not send message"))
            }
        }
    }
}

pub async fn create_channel() -> Result<EventChannel> {
    let producer = kafka::create_producer().await;

    Ok(EventChannel {
        _producer: producer,
    })
}

async fn process_event(owned_message: OwnedMessage) -> Result<(), KafkaError> {
    match extract_payload(owned_message) {
        Some(payload) => {
            let task_result = task::spawn_blocking(|| async move {
                println!("Received for processing: {}", payload);
                let storage = services::storage::initialize().await.unwrap();
                let result = services::image::generate(payload.as_str(), &storage).await;

                result
            })
            .await;

            match task_result {
                Ok(_) => {}
                Err(_) => {}
            };

            Ok(())
        }
        None => Ok(()),
    }
}

pub async fn consume_events(mut shutdown: Shutdown) {
    let consumer: StreamConsumer = kafka::create_consumer().await;
    let topic: String = String::from("image-optimizer");

    consumer
        .subscribe(&[&topic])
        .expect("Could not subscribe to topic: {:topic}");

    let stream = consumer
        .stream()
        .try_for_each(|borrowed_message| process_event(borrowed_message.detach()));

    println!("Listening to events for topic :{}", topic);

    select! {
        _ = stream =>
            println!("Stream Failed"),
        _ = &mut shutdown => {
            println!("Consumer shutdown");
        },
    }
}
