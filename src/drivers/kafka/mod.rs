use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::FutureProducer;
use rdkafka::Message;

pub async fn create_consumer() -> StreamConsumer {
    let group_id = String::from("optimisers");
    let brokers = String::from("localhost:9092");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", &brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Consumer creation failed");

    consumer
}

pub async fn create_producer() -> FutureProducer {
    let brokers = String::from("localhost:9092");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "6000")
        .create()
        .expect("Producer creation error");

    producer
}

pub fn extract_payload(message: OwnedMessage) -> Option<String> {
    match message.payload_view::<str>() {
        Some(Ok(payload)) => Some(payload.to_string()),
        Some(Err(_)) => None,
        None => None,
    }
}

// Backup of code from event service. TODO: If using figure out a better way to abstract kafka 
// related handling into this driver

pub async fn consume_events(mut shutdown: Shutdown) {
    let consumer: StreamConsumer = kafka::create_consumer().await;
    let topic: String = String::from("image-optimizer");

    consumer
        .subscribe(&[&topic])
        .expect("Could not subscribe to topic: {:topic}");

    let stream = consumer.stream().try_for_each(|borrowed_message| {
        let owned_message = borrowed_message.detach();
        match extract_payload(owned_message) {
            Some(payload) => process_event(payload),
            None => async { Ok(()) },
        }
    });

    log::info!("Listening to events for topic :{}", topic);

    select! {
        _ = stream =>
            log::error!("Stream Failed"),
        _ = &mut shutdown => {
            log::error!("Consumer shutdown");
        },
    }
}

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
                log::info!("Sent: {:?}", delivery);
                Ok(())
            }
            Err((e, _)) => {
                log::error!("Error: {:?}", e);
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

async fn process_event(message: String) -> Result<()> {
    let task_result = task::spawn_blocking(|| async move {
        log::info!("Received for processing: {}", message);
        let storage = services::storage::initialize().await.unwrap();
        let result = services::image::generate(message.as_str(), &storage).await;

        result
    })
    .await;

    match task_result {
        Ok(_) => {}
        Err(_) => {}
    };

    Ok(())
}
