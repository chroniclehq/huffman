use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub url: String,
}

pub fn deserialize(data: &str) -> Option<Message> {
    let result: Result<Message, serde_json::Error> = serde_json::from_str(data);
    match result {
        Ok(message) => Some(message),
        Err(error) => {
            log::warn!("{:?}", error);
            None
        }
    }
}

pub fn serialize(message: &Message) -> Option<String> {
    let result: Result<String, serde_json::Error> = serde_json::to_string(message);
    match result {
        Ok(data) => Some(data),
        Err(error) => {
            log::warn!("{:?}", error);
            None
        }
    }
}
