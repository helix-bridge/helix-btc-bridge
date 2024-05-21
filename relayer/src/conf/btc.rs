// crates.io
use bitcoin::Network;
use serde::{Deserialize, Serialize};
// self
use crate::chain::btc::{api::mempool::FeeType, types::*};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Conf {
	pub fee_conf: FeeConf,
	pub network: Network,
	pub secret_key: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FeeConf {
	pub extra: Satoshi,
	pub force: Option<Satoshi>,
	pub strategy: FeeType,
}
