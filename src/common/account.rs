use {
  crate::common::transaction::SanitizedTransaction,
  serde::{Deserialize, Serialize},
  solana_program::clock::Slot,
  solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfoV3,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountInfoV3 {
  pub slot: Slot,
  pub is_startup: bool,
  pub pubkey: Vec<u8>,
  pub lamports: u64,
  pub owner: Vec<u8>,
  pub executable: bool,
  pub rent_epoch: u64,
  pub data: Vec<u8>,
  pub write_version: u64,
  pub txn: Option<SanitizedTransaction>,
}

impl AccountInfoV3 {
  pub fn from(account_info: &ReplicaAccountInfoV3, slot: Slot, is_startup: bool) -> Self {
    Self {
      slot,
      is_startup,
      pubkey: account_info.pubkey.to_vec(),
      lamports: account_info.lamports,
      owner: account_info.owner.to_vec(),
      executable: account_info.executable,
      rent_epoch: account_info.rent_epoch,
      data: account_info.data.to_vec(),
      write_version: account_info.write_version,
      txn: match account_info.txn.to_owned() {
        Some(txn) => Some(SanitizedTransaction::from(txn)),
        None => None,
      },
    }
  }
}