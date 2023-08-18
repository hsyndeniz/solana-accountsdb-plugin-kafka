use log::info;
use serde::{Serialize, Deserialize};
use rdkafka::types::RDKafkaErrorCode;
use rdkafka::error::KafkaError;
use solana_sdk::reward_type::RewardType;
use solana_transaction_status::Reward;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotStatus {
    Processed,
    Rooted,
    Confirmed,
}

impl SlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SlotStatus::Confirmed => "confirmed",
            SlotStatus::Processed => "processed",
            SlotStatus::Rooted => "rooted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot: u64,
    pub parent: Option<u64>,
    pub status: SlotStatus,
}

impl SlotInfo {
    pub fn new(slot: u64, parent: Option<u64>, status: SlotStatus) -> Self {
        SlotInfo {
            slot,
            parent,
            status,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplicaBlockInfoV2<'a> {
    pub parent_slot: u64,
    pub parent_blockhash: &'a str,
    pub slot: u64,
    pub blockhash: &'a str,
    // pub rewards: &'a [Reward],
    pub block_time: Option<i64>,
    pub block_height: Option<u64>,
    pub executed_transaction_count: u64,
}

// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Reward {
//     pub pubkey: String,
//     pub lamports: i64,
//     pub post_balance: u64, // Account balance in lamports after `lamports` was applied
//     pub reward_type: Option<RewardType>,
//     pub commission: Option<u8>, // Vote account commission when the reward was credited, only present for voting and staking rewards
// }

pub fn serialize_slot_info(event: &SlotInfo) -> Result<String, KafkaError> {
    match serde_json::to_string(event) {
        Ok(v) => Ok(v),
        Err(e) => {
            info!("Failed to serialize slot status for slot {}: {:?}", event.slot, e);
            Err(KafkaError::MessageProduction(RDKafkaErrorCode::InvalidMessage))
        }
    }
}

pub fn serialize_block_info(event: &ReplicaBlockInfoV2) -> Result<String, KafkaError> {
    match serde_json::to_string(event) {
        Ok(v) => Ok(v),
        Err(e) => {
            info!("Failed to serialize block info for slot {}: {:?}", event.slot, e);
            Err(KafkaError::MessageProduction(RDKafkaErrorCode::InvalidMessage))
        }
    }
}

