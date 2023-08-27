use {
    serde::{Deserialize, Serialize},
    solana_geyser_plugin_interface::geyser_plugin_interface,
    solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaBlockInfoV2,
    solana_program::clock::Slot,
};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfoV2<'a> {
    pub parent_slot: u64,
    pub parent_blockhash: &'a str,
    pub slot: u64,
    pub blockhash: &'a str,
    pub rewards: Vec<solana_transaction_status::Reward>,
    pub block_time: Option<i64>,
    pub block_height: Option<u64>,
    pub executed_transaction_count: u64,
}

pub fn parse_slot_status(
    slot: Slot,
    parent: Option<u64>,
    status: geyser_plugin_interface::SlotStatus,
) -> SlotInfo {
    SlotInfo {
        slot,
        parent,
        status: match status {
            geyser_plugin_interface::SlotStatus::Confirmed => SlotStatus::Processed,
            geyser_plugin_interface::SlotStatus::Processed => SlotStatus::Confirmed,
            geyser_plugin_interface::SlotStatus::Rooted => SlotStatus::Rooted,
        },
    }
}

pub fn parse_block_metadata(block_metadata: ReplicaBlockInfoV2) -> BlockInfoV2 {
    BlockInfoV2 {
        parent_slot: block_metadata.parent_slot,
        parent_blockhash: &block_metadata.parent_blockhash,
        slot: block_metadata.slot,
        blockhash: &block_metadata.blockhash,
        rewards: block_metadata.rewards.to_vec(),
        block_time: block_metadata.block_time,
        block_height: block_metadata.block_height,
        executed_transaction_count: block_metadata.executed_transaction_count,
    }
}
