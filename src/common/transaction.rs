use {
    serde::{Deserialize, Serialize},
    solana_account_decoder::parse_token::{real_number_string, real_number_string_trimmed},
    solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2,
    solana_program::{
        clock::Slot,
        hash::Hash,
        message::{legacy, v0, AccountKeys},
    },
    solana_sdk::{
        signature::Signature, transaction::Result as TransactionResult, transaction::TransactionError, transaction_context::TransactionReturnData,
    },
    solana_transaction_status::{InnerInstructions, Rewards},
    std::borrow::Cow,
    std::str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfoV2 {
    pub slot: Slot,
    pub signature: Signature,
    pub is_vote: bool,
    pub transaction: SanitizedTransaction,
    pub transaction_status_meta: TransactionStatusMeta,
    pub index: usize,
}

impl TransactionInfoV2 {
    pub fn from(transaction: &ReplicaTransactionInfoV2, slot: Slot) -> Self {
        Self {
            slot,
            signature: Signature::from_str(&transaction.signature.to_string()).unwrap(),
            is_vote: transaction.is_vote,
            transaction: SanitizedTransaction::from(transaction.transaction),
            transaction_status_meta: TransactionStatusMeta {
                status: match transaction.transaction_status_meta.status.to_owned() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(TransactionError::from(e)),
                },
                fee: transaction.transaction_status_meta.fee,
                pre_balances: transaction.transaction_status_meta.pre_balances.to_vec(),
                post_balances: transaction.transaction_status_meta.post_balances.to_vec(),
                inner_instructions: transaction.transaction_status_meta.inner_instructions.to_owned(),
                log_messages: transaction.transaction_status_meta.log_messages.to_owned(),
                pre_token_balances: Some(
                    transaction
                        .transaction_status_meta
                        .pre_token_balances
                        .as_ref()
                        .map(|balances| balances.iter().map(|balance| TransactionTokenBalance::from(balance)).collect())
                        .unwrap_or_default(),
                ),
                post_token_balances: Some(
                    transaction
                        .transaction_status_meta
                        .post_token_balances
                        .as_ref()
                        .map(|balances| balances.iter().map(|balance| TransactionTokenBalance::from(balance)).collect())
                        .unwrap_or_default(),
                ),
                rewards: transaction.transaction_status_meta.rewards.to_owned(),
                loaded_addresses: transaction.transaction_status_meta.loaded_addresses.to_owned(),
                return_data: transaction.transaction_status_meta.return_data.to_owned(),
                compute_units_consumed: transaction.transaction_status_meta.compute_units_consumed,
            },
            index: transaction.index,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SanitizedTransaction {
    pub message: SanitizedMessage,
    pub message_hash: Hash,
    pub is_simple_vote_tx: bool,
    pub signatures: Vec<Signature>,
}

impl From<&solana_sdk::transaction::SanitizedTransaction> for SanitizedTransaction {
    fn from(transaction: &solana_sdk::transaction::SanitizedTransaction) -> Self {
        Self {
            message: match transaction.message() {
                solana_sdk::message::SanitizedMessage::Legacy(message) => SanitizedMessage::Legacy(LegacyMessage {
                    message: message.message.to_owned(),
                    is_writable_account_cache: message.is_writable_account_cache.to_vec(),
                }),
                solana_sdk::message::SanitizedMessage::V0(message) => SanitizedMessage::V0(LoadedMessage {
                    message: message.message.to_owned(),
                    loaded_addresses: message.loaded_addresses.to_owned(),
                    is_writable_account_cache: message.is_writable_account_cache.to_vec(),
                }),
            },
            message_hash: *transaction.message_hash(),
            is_simple_vote_tx: transaction.is_simple_vote_transaction(),
            signatures: transaction.signatures().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SanitizedMessage {
    Legacy(LegacyMessage<'static>),
    V0(LoadedMessage<'static>),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LegacyMessage<'a> {
    pub message: Cow<'a, legacy::Message>,
    pub is_writable_account_cache: Vec<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoadedMessage<'a> {
    pub message: Cow<'a, v0::Message>,
    pub loaded_addresses: Cow<'a, v0::LoadedAddresses>,
    pub is_writable_account_cache: Vec<bool>,
}

impl<'a> LegacyMessage<'a> {
    pub fn new(message: legacy::Message) -> Self {
        let is_writable_account_cache = message
            .account_keys
            .iter()
            .enumerate()
            .map(|(i, _key)| message.is_writable(i))
            .collect::<Vec<_>>();
        Self {
            message: Cow::Owned(message),
            is_writable_account_cache,
        }
    }

    pub fn has_duplicates(&self) -> bool {
        self.message.has_duplicates()
    }

    pub fn is_key_called_as_program(&self, key_index: usize) -> bool {
        self.message.is_key_called_as_program(key_index)
    }

    /// Inspect all message keys for the bpf upgradeable loader
    pub fn is_upgradeable_loader_present(&self) -> bool {
        self.message.is_upgradeable_loader_present()
    }

    /// Returns the full list of account keys.
    pub fn account_keys(&self) -> AccountKeys {
        AccountKeys::new(&self.message.account_keys, None)
    }

    pub fn is_writable(&self, index: usize) -> bool {
        *self.is_writable_account_cache.get(index).unwrap_or(&false)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionStatusMeta {
    pub status: TransactionResult<()>,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub inner_instructions: Option<Vec<InnerInstructions>>,
    pub log_messages: Option<Vec<String>>,
    pub pre_token_balances: Option<Vec<TransactionTokenBalance>>,
    pub post_token_balances: Option<Vec<TransactionTokenBalance>>,
    pub rewards: Option<Rewards>, // Vec<Reward>
    pub loaded_addresses: v0::LoadedAddresses,
    pub return_data: Option<TransactionReturnData>,
    pub compute_units_consumed: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionTokenBalance {
    pub account_index: u8,
    pub mint: String,
    pub ui_token_amount: UiTokenAmount,
    pub owner: String,
    pub program_id: String,
}

impl From<&solana_transaction_status::TransactionTokenBalance> for TransactionTokenBalance {
    fn from(balance: &solana_transaction_status::TransactionTokenBalance) -> Self {
        Self {
            account_index: balance.account_index,
            mint: balance.mint.to_string(),
            ui_token_amount: UiTokenAmount {
                ui_amount: balance.ui_token_amount.ui_amount,
                decimals: balance.ui_token_amount.decimals,
                amount: balance.ui_token_amount.amount.to_string(),
                ui_amount_string: balance.ui_token_amount.ui_amount_string.to_string(),
            },
            owner: balance.owner.to_string(),
            program_id: balance.program_id.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    pub ui_amount: Option<f64>,
    pub decimals: u8,
    pub amount: String,
    pub ui_amount_string: String,
}

impl Eq for UiTokenAmount {}

impl UiTokenAmount {
    pub fn real_number_string(&self) -> String {
        real_number_string(u64::from_str(&self.amount).unwrap_or_default(), self.decimals)
    }

    pub fn real_number_string_trimmed(&self) -> String {
        if !self.ui_amount_string.is_empty() {
            self.ui_amount_string.clone()
        } else {
            real_number_string_trimmed(u64::from_str(&self.amount).unwrap_or_default(), self.decimals)
        }
    }
}
