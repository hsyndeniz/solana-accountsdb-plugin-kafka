use {
    log::info,
    rdkafka::{
        producer::DefaultProducerContext,
        config::FromClientConfig,
        error::KafkaResult,
        producer::ThreadedProducer,
        ClientConfig,
    },
    serde::{Deserialize, Serialize},
    solana_geyser_plugin_interface::geyser_plugin_interface::{GeyserPluginError, Result},
    solana_sdk::commitment_config::CommitmentLevel,
    std::{
        collections::HashMap,
        fs::File,
        path::Path,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub kafka: HashMap<String, String>,
    pub shutdown_timeout_ms: u64,
    pub update_account_topic: String,
    pub transaction_topic: String,
    pub slot_status_topic: String,
    pub block_metadata_topic: String,
    pub entry_notification_topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub kafka_config: KafkaConfig,
    pub account_data_notifications_enabled: bool,
    pub transaction_notifications_enabled: bool,
    pub slot_notifications_enabled: bool,
    pub block_notifications_enabled: bool,
    pub entry_notifications_enabled: bool,
    pub commitment_level: CommitmentLevel,
    pub filters: Filters,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filters {
    pub accounts: AccountsFilter,
    pub transactions: Option<TransactionsFilter>,
    pub programs: Option<ProgramsFilter>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountsFilter {
    pub include: Vec<String>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgramsFilter {
    pub include: Vec<String>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionsFilter {
    pub mentions: Vec<String>,
}

impl Config {
    pub fn read_config<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        solana_logger::setup_with_default("solana=info");
        info!("Reading config file");
        let mut file = File::open(config_path).map_err(|err| {
            info!("Error opening config file: {}", err);
            GeyserPluginError::ConfigFileOpenError(err)
        })?;
        info!("Parsing config file: {:?}", file);
        let config: Config = serde_json::from_reader(&mut file).map_err(|err| {
            info!("Error reading config file: {}", err);
            GeyserPluginError::ConfigFileReadError { msg: err.to_string() }
        })?;
        Ok(config)
    }

    pub fn producer(&self) -> KafkaResult<ThreadedProducer<DefaultProducerContext>> {
        let mut config = ClientConfig::new();
        for (k, v) in self.kafka_config.kafka.iter() {
            config.set(k, v);
        }
        ThreadedProducer::from_config(&config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kafka_config: KafkaConfig {
                kafka: HashMap::new(),
                shutdown_timeout_ms: 30000,
                update_account_topic: "update_account".to_string(),
                transaction_topic: "transaction".to_string(),
                slot_status_topic: "slot_status".to_string(),
                block_metadata_topic: "block_metadata".to_string(),
                entry_notification_topic: "entry_notification".to_string(),
            },
            account_data_notifications_enabled: true,
            transaction_notifications_enabled: true,
            slot_notifications_enabled: true,
            block_notifications_enabled: true,
            entry_notifications_enabled: true,
            commitment_level: CommitmentLevel::Processed,
            filters: Filters {
                accounts: AccountsFilter {
                    include: vec![],
                    exclude: None,
                },
                transactions: None,
                programs: None,
            },
        }
    }
}
