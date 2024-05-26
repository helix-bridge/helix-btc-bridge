// crates.io
use bitcoin::Network;
use serde::{Deserialize, Serialize};
// self
use crate::chain::btc::{api::mempool::FeeType, types::*};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Conf {
	pub network: Network,
	pub vault_secret_key: String,
	pub fee_conf: FeeConf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FeeConf {
	pub strategy: FeeType,
	pub extra: Satoshi,
	pub force: Option<Satoshi>,
}
