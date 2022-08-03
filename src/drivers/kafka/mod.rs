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
        Some(Err(_)) => Some("".to_string()),
        None => None,
    }
}
