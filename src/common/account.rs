use log::info;
use solana_account_decoder::parse_account_data::ParsedAccount;
use {
    crate::common::transaction::SanitizedTransaction,
    serde::{Deserialize, Serialize},
    solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfoV3,
    solana_program::clock::Slot,
    solana_program::pubkey::Pubkey,
    solana_account_decoder::parse_account_data::parse_account_data,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountInfoV3 {
    pub slot: Slot,
    pub is_startup: bool,
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
    pub parsed_data: Option<ParsedAccount>,
    pub write_version: u64,
    pub txn: Option<SanitizedTransaction>,
}

impl AccountInfoV3 {
    pub fn from(account_info: &ReplicaAccountInfoV3, slot: Slot, is_startup: bool) -> Self {
        let pubkey = Pubkey::try_from(account_info.pubkey.clone()).unwrap();
        let owner = Pubkey::try_from(account_info.owner.clone()).unwrap();
        let txn = match account_info.txn.to_owned() {
            Some(txn) => Some(SanitizedTransaction::from(txn)),
            None => None,
        };
        let account_data = parse_account_data(&pubkey, &owner, &account_info.data, None);
        let parsed_data = match account_data {
            Ok(parsed_data) => Some(parsed_data),
            Err(e) => {
                info!("Error parsing account data: {:?}", e);
                None
            }
        };
        Self {
            slot,
            is_startup,
            pubkey: pubkey.to_string(),
            lamports: account_info.lamports,
            owner: owner.to_string(),
            executable: account_info.executable,
            rent_epoch: account_info.rent_epoch,
            data: account_info.data.to_vec(),
            parsed_data,
            write_version: account_info.write_version,
            txn,
        }
    }
}
