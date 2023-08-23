use serde::Serialize;
use rdkafka::error::{KafkaError, RDKafkaErrorCode};

pub mod block;
pub mod account;
pub mod transaction;

pub fn serialize_struct<T: Serialize>(s: T) -> Result<String, KafkaError> {
  match serde_json::to_string(&s) {
    Ok(v) => Ok(v),
    Err(e) => Err(KafkaError::MessageProduction(RDKafkaErrorCode::InvalidMessage))
  }
}