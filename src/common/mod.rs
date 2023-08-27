use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use serde::Serialize;

pub mod account;
pub mod block;
pub mod transaction;

pub fn serialize_struct<T: Serialize>(s: T) -> Result<String, KafkaError> {
    match serde_json::to_string(&s) {
        Ok(v) => Ok(v),
        Err(_e) => Err(KafkaError::MessageProduction(RDKafkaErrorCode::InvalidMessage)),
    }
}
