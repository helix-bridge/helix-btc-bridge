// std
use std::{
	fs,
	path::{Path, PathBuf},
	process,
};
// crates.io
use app_dirs2::{AppDataType, AppInfo};
use bitcoin::{key::Keypair, secp256k1::Secp256k1, Network};
use serde::{Deserialize, Serialize};
// self
use super::*;
use crate::api::Api;

const APP_INFO: AppInfo = AppInfo { name: "helix-btc-bridge-relayer", author: "Xavier Lau" };
const DEFAULT_CONF: &str = r#"# Network configuration.
# Possible values: "mainnet", "testnet", "signet", "regtest".
network = "testnet"

# Secret key in hex format (optional "0x" prefix).
secret-key = ""

[fee-conf]
# Additional fee to add to the recommended fee rate (in satoshis per byte).
extra = 0

# Force set the fee rate (in satoshis per byte).
# force = 1

# Fee strategy to use for transactions.
# Possible values (sorted from fastest to slowest): "fastest", "half-hour", "hour", "economy", "minimum".
strategy = "fastest""#;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RelayerConf {
	fee_conf: FeeConf,
	network: Network,
	secret_key: String,
}
impl RelayerConf {
	fn default_path() -> Result<PathBuf> {
		Ok(app_dirs2::app_root(AppDataType::UserConfig, &APP_INFO)?.join("conf.toml"))
	}

	fn load_from(path: &Path) -> Result<Self> {
		if path.is_file() {
			let s = fs::read_to_string(path)?;

			match toml::from_str(&s) {
				Ok(r) => Ok(r),
				Err(e) => {
					tracing::error!(
						"an error occurred while parsing the configuration, \
						please check the {path:?}",
					);

					Err(e)?
				},
			}
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
impl Default for RelayerConf {
	fn default() -> Self {
		toml::from_str(DEFAULT_CONF).unwrap()
	}
}
impl TryFrom<Relayer> for RelayerConf {
	type Error = Error;

	fn try_from(value: Relayer) -> Result<Self> {
		Ok(Self {
			fee_conf: value.fee_conf,
			network: value.network,
			secret_key: format!("0x{}", value.keypair.display_secret()),
		})
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FeeConf {
	pub extra: Satoshi,
	pub force: Option<Satoshi>,
	pub strategy: FeeStrategy,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeeStrategy {
	Economy,
	Fastest,
	HalfHour,
	Hour,
	Minimum,
}
impl Default for FeeStrategy {
	fn default() -> Self {
		Self::Fastest
	}
}

impl Relayer {
	pub fn load() -> Result<Self> {
		let p = RelayerConf::default_path()?;

		match Relayer::try_from(RelayerConf::load_from(&p)?) {
			Ok(r) => {
				Api::init(matches!(r.network, Network::Testnet))?;

				Ok(r)
			},
			r => {
				tracing::error!(
					"an error occurred while parsing the configuration, \
					please check the {p:?}",
				);

				r
			},
		}
	}
}
impl TryFrom<RelayerConf> for Relayer {
	type Error = Error;

	fn try_from(value: RelayerConf) -> Result<Self> {
		Ok(Self {
			fee_conf: value.fee_conf,
			keypair: Keypair::from_seckey_str(
				&Secp256k1::new(),
				value.secret_key.trim_start_matches("0x"),
			)?,
			network: value.network,
		})
	}
}
