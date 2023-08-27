// Copyright 2022 Blockdaemon Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use {
    crate::{
        common::block::{BlockInfoV2, SlotInfo},
        common::serialize_struct,
        message_wrapper::EventMessage::{self, Account, Transaction},
        prom::{
            StatsThreadedProducerContext, UPLOAD_ACCOUNTS_TOTAL, UPLOAD_SLOTS_TOTAL,
            UPLOAD_TRANSACTIONS_TOTAL,
        },
        Config, MessageWrapper, TransactionEvent, UpdateAccountEvent,
    },
    log::info,
    prost::Message,
    rdkafka::{
        error::{KafkaError, RDKafkaErrorCode},
        producer::{BaseRecord, Producer, ThreadedProducer},
    },
    std::time::Duration,
};

pub struct Publisher {
    producer: ThreadedProducer<StatsThreadedProducerContext>,
    shutdown_timeout: Duration,

    update_account_topic: String,
    slot_status_topic: String,
    transaction_topic: String,
    entry_notification_topic: String,
    block_metadata_topic: String,
}

impl Publisher {
    pub fn new(producer: ThreadedProducer<StatsThreadedProducerContext>, config: &Config) -> Self {
        Self {
            producer,
            shutdown_timeout: Duration::from_millis(config.kafka_config.shutdown_timeout_ms),
            update_account_topic: config.kafka_config.update_account_topic.clone(),
            slot_status_topic: config.kafka_config.slot_status_topic.clone(),
            transaction_topic: config.kafka_config.transaction_topic.clone(),
            entry_notification_topic: config.kafka_config.entry_notification_topic.clone(),
            block_metadata_topic: config.kafka_config.block_metadata_topic.clone(),
        }
    }

    pub fn update_account(&self, ev: UpdateAccountEvent) -> Result<(), KafkaError> {
        let (key, buf) = (&ev.pubkey, ev.encode_to_vec());
        let record = BaseRecord::<Vec<u8>, _>::to(&self.update_account_topic)
            .key(key)
            .payload(&buf);
        let result = self.producer.send(record).map(|_| ()).map_err(|(e, _)| e);
        UPLOAD_ACCOUNTS_TOTAL
            .with_label_values(&[if result.is_ok() { "success" } else { "failed" }])
            .inc();
        result
    }

    pub fn update_slot_status(&self, event: SlotInfo) -> Result<(), KafkaError> {
        solana_logger::setup_with_default("info");

        let _key = event.slot.to_string();
        // let value = serialize_slot_info(&event).unwrap();
        //
        // let record = BaseRecord::<String, _>::to(&self.slot_status_topic)
        //     .key(&key)
        //     .payload(&value);
        //
        // match self.producer.send(record) {
        //     Ok(_) => {
        //         UPLOAD_SLOTS_TOTAL.with_label_values(&["success"]).inc();
        //         Ok(())
        //     }
        //     Err((e, _)) => {
        //         info!(
        //             "Failed to send slot status for slot {}: {:?}",
        //             event.slot, e
        //         );
        //         UPLOAD_SLOTS_TOTAL.with_label_values(&["failed"]).inc();
        //         Err(KafkaError::MessageProduction(RDKafkaErrorCode::Fail))
        //     }
        // }
        Ok(())
    }

    pub fn update_transaction(&self, ev: TransactionEvent) -> Result<(), KafkaError> {
        let (key, buf) = (&ev.signature, ev.encode_to_vec());
        let record = BaseRecord::<Vec<u8>, _>::to(&self.transaction_topic)
            .key(key)
            .payload(&buf);
        let result = self.producer.send(record).map(|_| ()).map_err(|(e, _)| e);
        UPLOAD_TRANSACTIONS_TOTAL
            .with_label_values(&[if result.is_ok() { "success" } else { "failed" }])
            .inc();
        result
    }

    pub fn update_entry(&self, _event: String) -> Result<(), KafkaError> {
        info!("update_entry");
        return Ok(());
    }

    pub fn update_block_metadata(&self, event: &BlockInfoV2) -> Result<(), KafkaError> {
        solana_logger::setup_with_default("info");
        info!(
            "update_block_metadata, topic: {}",
            self.block_metadata_topic
        );
        let key = event.slot.to_string();
        let value = serialize_struct(&event).unwrap();

        let record = BaseRecord::<String, _>::to(&self.block_metadata_topic)
            .key(&key)
            .payload(&value);

        match self.producer.send(record) {
            Ok(_) => {
                info!("block metadata sent to kafka");
                Ok(())
            }
            Err((e, _)) => {
                info!(
                    "Failed to send block metadata for slot {}: {:?}",
                    event.slot, e
                );
                Err(KafkaError::MessageProduction(RDKafkaErrorCode::Fail))
            }
        }
    }

    pub fn wants_update_account(&self) -> bool {
        !self.update_account_topic.is_empty()
    }

    pub fn wants_slot_status(&self) -> bool {
        !self.slot_status_topic.is_empty()
    }

    pub fn wants_transaction(&self) -> bool {
        !self.transaction_topic.is_empty()
    }

    pub fn wants_entry_notification(&self) -> bool {
        !self.entry_notification_topic.is_empty()
    }

    pub fn wants_block_metadata(&self) -> bool {
        !self.block_metadata_topic.is_empty()
    }

    fn encode_with_wrapper(message: EventMessage) -> Vec<u8> {
        MessageWrapper {
            event_message: Some(message),
        }
        .encode_to_vec()
    }

    fn copy_and_prepend(&self, data: &[u8], prefix: u8) -> Vec<u8> {
        let mut temp_key = Vec::with_capacity(data.len() + 1);
        temp_key.push(prefix);
        temp_key.extend_from_slice(data);
        temp_key
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let _ = self.producer.flush(self.shutdown_timeout);
    }
}
