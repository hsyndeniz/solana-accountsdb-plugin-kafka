use {
    crate::{
        common::account::AccountInfoV3,
        common::block::{parse_block_metadata, parse_entry_info, parse_slot_status},
        common::transaction::TransactionInfoV2,
        config::Config,
        KafkaService,
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
        ReplicaEntryInfoVersions, ReplicaTransactionInfoVersions, Result, SlotStatus,
    },
    solana_program::clock::Slot,
    solana_program::pubkey::Pubkey,
    solana_sdk::commitment_config::CommitmentLevel,
    log::{error, info},
    std::fmt::Debug,
};

#[derive(Default)]
pub struct IndexerPlugin {
    kafka: Option<KafkaService>,
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

        let kafka = KafkaService::new(producer, &config);
        self.kafka = Some(kafka);
        Ok(())
    }

    fn on_unload(&mut self) {
        info!("Unloading solana-indexer-geyser-plugin");
        todo!("Shutdown any threads, services, etc. here");
    }

    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: Slot,
        is_startup: bool,
    ) -> Result<()> {
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
                if !self.account_data_notifications_enabled {
                    // * ignoring account update: account data notifications disabled
                    return Ok(());
                }
                let pubkey = Pubkey::try_from(_info.pubkey.clone()).unwrap().to_string();
                // if pubkey starts with Sysvar, ignore
                if pubkey.starts_with("Sysvar") {
                    info!("ignoring account update: system account");
                    return Ok(());
                }

                match _info.txn.to_owned() {
                    Some(txn) => {
                        if txn.is_simple_vote_transaction() {
                            info!("ignoring account update: vote transaction");
                            return Ok(());
                        }
                        Some(txn)
                    }
                    None => None,
                };

                let account_info = AccountInfoV3::from(_info, slot, is_startup);
                info!("account_info: slot: {:?}, pubkey: {:?}, is_startup: {:?}, lamports: {:?}, owner: {:?}, executable: {:?}, rent_epoch: {:?}, write_version: {:?}, txn: {:?}", account_info.slot, account_info.pubkey, account_info.is_startup, account_info.lamports, account_info.owner, account_info.executable, account_info.rent_epoch, account_info.write_version, account_info.txn);
                self.kafka().update_account(account_info).map_err(|error| {
                    GeyserPluginError::AccountsUpdateError {
                        msg: error.to_string(),
                    }
                })
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
            // * ignoring slot status: commitment level not met
            return Ok(());
        }

        let slot_info = parse_slot_status(slot, parent, status);
        info!("slot_info: {:?}", slot_info);
        self.kafka().update_slot_status(slot_info).map_err(|error| {
            GeyserPluginError::SlotStatusUpdateError {
                msg: error.to_string(),
            }
        })
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: Slot,
    ) -> Result<()> {
        match transaction {
            ReplicaTransactionInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaTransactionInfoVersion::V0_0_1 unsupported, please upgrade your Solana node.");
                // ? Clarify panic and what to do
            }
            ReplicaTransactionInfoVersions::V0_0_2(_info) => {
                if !self.transaction_notifications_enabled {
                    // * ignoring transaction notification: transaction notifications disabled
                    return Ok(());
                }
                // ? Ignore vote transactions
                let transaction_info = TransactionInfoV2::from(_info, slot);
                info!("transaction_info: {:?}", transaction_info);
                if transaction_info.is_vote {
                    return Ok(());
                }
                self.kafka()
                    .update_transaction(transaction_info)
                    .map_err(|error| GeyserPluginError::TransactionUpdateError {
                        msg: error.to_string(),
                    })
            }
        }
    }

    fn notify_entry(&self, entry: ReplicaEntryInfoVersions) -> Result<()> {
        match entry {
            ReplicaEntryInfoVersions::V0_0_1(_info) => {
                if !self.entry_notifications_enabled {
                    // * ignoring entry notification: entry notifications disabled
                    return Ok(());
                } else {
                    let entry_info = parse_entry_info(_info.clone());
                    info!("entry_info: {:?}", entry_info);
                    Ok(())
                    // self.kafka()
                    //     .update_entry_notification(entry_info)
                    //     .map_err(|error| GeyserPluginError::SlotStatusUpdateError {
                    //         msg: error.to_string(),
                    //     })
                }
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
                if !self.block_notifications_enabled {
                    // * ignoring block notification: block notifications disabled
                    return Ok(());
                }
                let block_info = parse_block_metadata(_info.clone());
                info!("parsed_block_info: {:?}", block_info);
                self.kafka()
                    .update_block_metadata(block_info)
                    .map_err(|error| GeyserPluginError::SlotStatusUpdateError {
                        msg: error.to_string(),
                    })
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

    pub fn kafka(&self) -> &KafkaService {
        self.kafka.as_ref().expect("Kafka service not initialized")
    }
}
