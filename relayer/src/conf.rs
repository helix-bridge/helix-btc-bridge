pub mod btc;

// std
use std::{fs, path::Path, process};
// crates.io
use serde::{Deserialize, Serialize};
// self
use crate::prelude::*;

const DEFAULT_CONF: &str = r#"[btc]
# Network configuration.
# Possible values: "mainnet", "testnet", "signet", "regtest".
network = "testnet"

# Vault secret key in hex format (optional "0x" prefix).
vault-secret-key = "0x.."

[btc.fee-conf]
# Fee strategy to use for transactions.
# Possible values (sorted from fastest to slowest): "fastest", "half-hour", "hour", "economy", "minimum".
strategy = "fastest"

# Additional fee to add to the recommended fee rate (in satoshis per byte).
extra = 0

# Force set the fee rate (in satoshis per byte).
# force = 1
"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct Conf {
	pub btc: btc::Conf,
	// pub ckb: ckb::Conf,
}
impl Conf {
	pub fn load_from(path: &Path) -> Result<Self> {
		if path.is_file() {
			Ok(toml::from_str(&fs::read_to_string(path)?)?)
		} else {
			tracing::info!(
				"no configuration file found, \
				use the template to generate a new one, \
				please configure it at {path:?}"
			);
			fs::write(path, DEFAULT_CONF)?;
			process::exit(0);
		}
	}
}
impl Default for Conf {
	fn default() -> Self {
		toml::from_str(DEFAULT_CONF).unwrap()
	}
}
