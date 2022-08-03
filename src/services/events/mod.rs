use crate::drivers::kafka::{self, extract_payload};
use anyhow::{anyhow, Result};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rocket::futures::TryStreamExt;
use rocket::tokio::{select, task, time};
use rocket::Shutdown;

pub struct Events {
    _producer: FutureProducer,
    _consumer: StreamConsumer,
}

impl Events {
    // TODO: Move this into a fn compatible with rocket
    #[allow(dead_code)]
    pub async fn send(&self, key: &str, payload: &str) -> Result<()> {
        let topic = String::from("image-optimizer");

        let record = FutureRecord::to(&topic).key(key).payload(payload);
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

pub async fn initialize() -> Result<Events> {
    let consumer = kafka::create_consumer().await;
    let producer = kafka::create_producer().await;

    Ok(Events {
        _consumer: consumer,
        _producer: producer,
    })
}

pub async fn start_consumer(
    eventbus: Events,
    process: fn(String) -> Result<()>,
    mut shutdown: Shutdown,
) {
    let topic = String::from("image-optimizer");

    eventbus
        ._consumer
        .subscribe(&[&topic])
        .expect("Could not subscribe to topic: {:topic}");

    let stream = eventbus
        ._consumer
        .stream()
        .try_for_each(|borrowed_message| {
            let owned_message = borrowed_message.detach();

            async move {
                let _: Result<()> = match extract_payload(owned_message) {
                    Some(payload) => {
                        let _ = task::spawn_blocking(move || {
                            let _ = process(payload);
                        })
                        .await;

                        Ok(())
                    }
                    None => Ok(()),
                };
                Ok(())
            }
        });

    println!("Listening to events for topic :{}", topic);

    select! {
        _ = stream =>
            println!("Stream shutdown"),
        _ = &mut shutdown => {
            println!("Shutting down listener");
        },
    }
}
