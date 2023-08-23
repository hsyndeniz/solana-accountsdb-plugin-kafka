use {
    crate::{
        common::transaction::TransactionInfoV2,
        common::block::{parse_block_metadata, parse_slot_status},
        common::account::AccountInfoV3,
        config::Config,
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPlugin, GeyserPluginError, Result, SlotStatus, ReplicaBlockInfoVersions,
        ReplicaEntryInfoVersions, ReplicaTransactionInfoVersions, ReplicaAccountInfoVersions,
    },
    std::fmt::Debug,
    log::{error, info},
    rdkafka::producer::Producer,
    solana_program::clock::Slot,
    solana_sdk::commitment_config::CommitmentLevel,
};

#[derive(Default)]
pub struct IndexerPlugin {
    commitment_level: CommitmentLevel,
    account_data_notifications_enabled: bool,
    transaction_notifications_enabled: bool,
    slot_notifications_enabled: bool,
    block_notifications_enabled: bool,
    entry_notifications_enabled: bool,
}

impl Debug for IndexerPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexerPlugin")
            .field("commitment_level", &self.commitment_level)
            .field(
                "account_data_notifications_enabled",
                &self.account_data_notifications_enabled,
            )
            .field(
                "transaction_notifications_enabled",
                &self.transaction_notifications_enabled,
            )
            .field(
                "slot_notifications_enabled",
                &self.slot_notifications_enabled,
            )
            .field(
                "block_notifications_enabled",
                &self.block_notifications_enabled,
            )
            .field(
                "entry_notifications_enabled",
                &self.entry_notifications_enabled,
            )
            // possible to add more fields here
            .finish()
    }
}

impl GeyserPlugin for IndexerPlugin {
    fn name(&self) -> &'static str {
        "solana-indexer-geyser-plugin"
    }

    fn on_load(&mut self, _config_file: &str) -> Result<()> {
        solana_logger::setup_with_default("info");
        info!("Loading solana-indexer-geyser-plugin from {}", _config_file);

        let config = Config::read_config(_config_file)?;

        self.account_data_notifications_enabled = config.account_data_notifications_enabled;
        self.transaction_notifications_enabled = config.transaction_notifications_enabled;
        self.slot_notifications_enabled = config.slot_notifications_enabled;
        self.block_notifications_enabled = config.block_notifications_enabled;
        self.entry_notifications_enabled = config.entry_notifications_enabled;
        self.commitment_level = CommitmentLevel::from(config.commitment_level);

        let producer = config.producer().map_err(|error| {
            error!("Failed to create kafka producer: {error:?}");
            GeyserPluginError::Custom(Box::new(error))
        })?;
        info!("Created rdkafka::FutureProducer");
        info!("producer: {:?}", producer.context());
        Ok(())
    }

    fn on_unload(&mut self) {
        info!("Unloading solana-indexer-geyser-plugin");
        todo!("Shutdown any threads, services, etc. here");
    }

    fn update_account(&self, account: ReplicaAccountInfoVersions, slot: Slot, is_startup: bool) -> Result<()> {
        match account {
            ReplicaAccountInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaAccountInfoVersion::V0_0_1 unsupported, please upgrade your Solana node.");
                // ? Clarify panic and what to do
            }
            ReplicaAccountInfoVersions::V0_0_2(_info) => {
                panic!("ReplicaAccountInfoVersion::V0_0_2 unsupported, please upgrade your Solana node.");
                // ? Clarify panic and what to do
            }
            ReplicaAccountInfoVersions::V0_0_3(_info) => {
                let account_info = AccountInfoV3::from(_info, slot, is_startup);
                info!("account_info: {:?}", account_info);
                // ? Send account info to kafka and/or postgres
                Ok(())
            }
        }
    }

    fn notify_end_of_startup(&self) -> Result<()> {
        todo!()
    }

    fn update_slot_status(
        &self,
        slot: Slot,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> Result<()> {
        let commitment_level = match status {
            SlotStatus::Processed => CommitmentLevel::Processed,
            SlotStatus::Confirmed => CommitmentLevel::Confirmed,
            SlotStatus::Rooted => CommitmentLevel::Finalized,
        };

        if commitment_level != self.commitment_level {
            // * ignoring slot status update because of commitment level
            return Ok(());
        }

        let slot_info = parse_slot_status(slot, parent, status);
        info!("slot_info: {:?}", slot_info);
        // ? Send slot status to kafka and/or postgres
        Ok(())
    }

    fn notify_transaction(&self, transaction: ReplicaTransactionInfoVersions, slot: Slot) -> Result<()> {
        match transaction {
            ReplicaTransactionInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaTransactionInfoVersion::V0_0_1 unsupported, please upgrade your Solana node.");
                // ? Clarify panic and what to do
            }
            ReplicaTransactionInfoVersions::V0_0_2(_info) => {
                let transaction_info = TransactionInfoV2::from(_info, slot);
                info!("transaction_info: {:?}", transaction_info);
                // ? Send transaction info to kafka and/or postgres
                Ok(())
            }
        }
    }

    fn notify_entry(&self, entry: ReplicaEntryInfoVersions) -> Result<()> {
        match entry {
            ReplicaEntryInfoVersions::V0_0_1(_info) => {
                // ? Clarify if ReplicaEntryInfo needs to be indexed
                Ok(())
            }
        }
    }

    fn notify_block_metadata(&self, blockinfo: ReplicaBlockInfoVersions) -> Result<()> {
        match blockinfo {
            ReplicaBlockInfoVersions::V0_0_1(_info) => {
                panic!(
                    "ReplicaBlockInfoVersion::V0_0_1 unsupported, please upgrade your Solana node."
                );
                // ? Clarify panic and what to do
            }
            ReplicaBlockInfoVersions::V0_0_2(_info) => {
                let block_info = parse_block_metadata(_info.clone());
                info!("parsed_block_info: {:?}", block_info);
                info!("rewards: {:?}", block_info.rewards);
                // ? Send block metadata to kafka and/or postgres
                Ok(())
            }
        }
    }

    fn account_data_notifications_enabled(&self) -> bool {
        self.account_data_notifications_enabled
    }

    fn transaction_notifications_enabled(&self) -> bool {
        self.transaction_notifications_enabled
    }

    fn entry_notifications_enabled(&self) -> bool {
        self.entry_notifications_enabled
    }
}

impl IndexerPlugin {
    pub fn new() -> Self {
        Default::default()
    }
}
