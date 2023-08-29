use {
    crate::{
        common::{
            account::AccountInfoV3,
            block::{BlockInfoV2, EntryInfo, SlotInfo},
            serialize_struct,
            transaction::TransactionInfoV2,
        },
        Config,
    },
    log::info,
    rdkafka::{
        error::{KafkaError, RDKafkaErrorCode},
        producer::{BaseRecord, DefaultProducerContext, Producer, ThreadedProducer},
    },
    std::time::Duration,
};

pub struct KafkaService {
    producer: ThreadedProducer<DefaultProducerContext>,
    shutdown_timeout: Duration,
    update_account_topic: String,
    transaction_topic: String,
    slot_status_topic: String,
    block_metadata_topic: String,
    entry_notification_topic: String,
}

impl KafkaService {
    pub fn new(producer: ThreadedProducer<DefaultProducerContext>, config: &Config) -> Self {
        Self {
            producer,
            shutdown_timeout: Duration::from_millis(config.kafka_config.shutdown_timeout_ms),
            update_account_topic: config.kafka_config.update_account_topic.clone(),
            transaction_topic: config.kafka_config.transaction_topic.clone(),
            slot_status_topic: config.kafka_config.slot_status_topic.clone(),
            block_metadata_topic: config.kafka_config.block_metadata_topic.clone(),
            entry_notification_topic: config.kafka_config.entry_notification_topic.clone(),
        }
    }

    fn send_to_topic<T>(&self, topic: &str, key: String, item: T) -> Result<(), KafkaError>
    where
        T: serde::Serialize,
    {
        match serialize_struct(&item) {
            Ok(value) => {
                let record = BaseRecord::to(topic).key(&key).payload(&value);
                self.producer.send(record).map(|_| ()).map_err(|(e, _)| e)
            }
            Err(e) => {
                info!("serialize_struct error: {:?}", e);
                Err(KafkaError::MessageProduction(
                    RDKafkaErrorCode::InvalidMessage,
                ))
            }
        }
    }

    pub fn update_account(&self, account_info: AccountInfoV3) -> Result<(), KafkaError> {
        self.send_to_topic(
            &self.update_account_topic,
            account_info.pubkey.clone(),
            account_info,
        )
    }

    pub fn update_transaction(
        &self,
        transaction_info: TransactionInfoV2,
    ) -> Result<(), KafkaError> {
        self.send_to_topic(
            &self.transaction_topic,
            transaction_info.signature.to_string(),
            transaction_info,
        )
    }

    pub fn update_slot_status(&self, slot_info: SlotInfo) -> Result<(), KafkaError> {
        self.send_to_topic(
            &self.slot_status_topic,
            slot_info.slot.to_string(),
            slot_info,
        )
    }

    pub fn update_block_metadata(&self, block_info: BlockInfoV2) -> Result<(), KafkaError> {
        self.send_to_topic(
            &self.block_metadata_topic,
            block_info.blockhash.to_string(),
            block_info,
        )
    }

    pub fn update_entry_notification(&self, entry_info: EntryInfo) -> Result<(), KafkaError> {
        self.send_to_topic(
            &self.entry_notification_topic,
            entry_info.slot.to_string(),
            entry_info,
        )
    }
}

impl Drop for KafkaService {
    fn drop(&mut self) {
        let _ = self.producer.flush(self.shutdown_timeout);
    }
}
