use {
    std::borrow::Cow,
    serde::{Deserialize, Serialize},
    solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2,
    solana_program::{
        clock::Slot,
        pubkey::Pubkey,
        message::{legacy, v0, AccountKeys},
        instruction::CompiledInstruction,
    },
    solana_transaction_status::{
        UiInstruction, UiParsedInstruction, UiPartiallyDecodedInstruction, UiTransactionStatusMeta,
        parse_accounts::{parse_legacy_message_accounts, parse_v0_message_accounts},
        parse_instruction::parse,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfoV2 {
    pub slot: Slot,
    pub signature: String,
    pub is_vote: bool,
    pub transaction: SanitizedTransaction,
    pub transaction_status_meta: UiTransactionStatusMeta,
    pub index: usize,
}

impl TransactionInfoV2 {
    pub fn from(transaction: &ReplicaTransactionInfoV2, slot: Slot) -> Self {
        Self {
            slot,
            signature: transaction.signature.to_owned().to_string(),
            is_vote: transaction.is_vote,
            transaction: SanitizedTransaction::from(transaction.transaction),
            transaction_status_meta: UiTransactionStatusMeta::from(transaction.transaction_status_meta.clone()),
            index: transaction.index,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SanitizedTransaction {
    pub message_hash: String,
    pub is_simple_vote_tx: bool,
    pub signatures: Vec<String>,
    pub message: Option<solana_transaction_status::UiParsedMessage>,
}

impl From<&solana_sdk::transaction::SanitizedTransaction> for SanitizedTransaction {
    fn from(transaction: &solana_sdk::transaction::SanitizedTransaction) -> Self {
        Self {
            message: match transaction.message() {
                solana_sdk::message::SanitizedMessage::Legacy(message) => {
                    let account_keys = AccountKeys::new(&message.message.account_keys, None);
                    Some(solana_transaction_status::UiParsedMessage {
                        account_keys: parse_legacy_message_accounts(&message.message),
                        recent_blockhash: message.message.recent_blockhash.to_string(),
                        instructions: parse_instructions(message.message.instructions.clone(), account_keys, message.message.account_keys.clone()),
                        address_table_lookups: None,
                    })
                }
                solana_sdk::message::SanitizedMessage::V0(message) => {
                    let account_keys = AccountKeys::new(&message.message.account_keys, None);
                    Some(solana_transaction_status::UiParsedMessage {
                        account_keys: parse_v0_message_accounts(&message),
                        recent_blockhash: message.message.recent_blockhash.to_string(),
                        instructions: parse_instructions(message.message.instructions.clone(), account_keys, message.message.account_keys.clone()),
                        address_table_lookups: Some(message.message.address_table_lookups.iter().map(core::convert::Into::into).collect())
                    })
                }
            },
            message_hash: format!("{}", transaction.message_hash()),
            is_simple_vote_tx: transaction.is_simple_vote_transaction(),
            signatures: transaction.signatures().iter().map(|s| s.to_string()).collect(),
        }
    }
}

pub fn parse_instructions(
    instructions: Vec<CompiledInstruction>,
    account_keys: AccountKeys,
    pub_keys: Vec<Pubkey>
) -> Vec<UiInstruction> {
    instructions.iter().map(|i| {
        let program_id = &account_keys[i.program_id_index as usize];
        let account_keys = AccountKeys::new(&*pub_keys, None);
        if let Ok(parsed_instruction) = parse(program_id, i, &account_keys, None) {
            UiInstruction::Parsed(UiParsedInstruction::Parsed(parsed_instruction))
        } else {
            UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
                UiPartiallyDecodedInstruction {
                    program_id: account_keys[i.program_id_index as usize].to_string(),
                    accounts: i
                        .accounts
                        .iter()
                        .map(|&i| account_keys[i as usize].to_string())
                        .collect(),
                    data: bs58::encode(i.data.clone()).into_string(),
                    stack_height: None,
                },
            ))
        }
    }).collect()
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
