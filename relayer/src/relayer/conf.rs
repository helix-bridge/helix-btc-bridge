// std
use std::{fs, path::PathBuf};
// crates.io
use app_dirs2::{AppDataType, AppInfo};
use bitcoin::{
	key::Keypair,
	secp256k1::{rand, Secp256k1},
	Network,
};
use serde::{Deserialize, Serialize};
// self
use super::*;

const APP_INFO: AppInfo = AppInfo { name: "helix-btc-bridge-relayer", author: "Xavier Lau" };
// const DEFAULT_CONF: &str =
// r#"seed = ""
// rpc = ""

// "#;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct RelayerConf {
	pub(super) seed: String,
	pub(super) network: Network,
}
impl RelayerConf {
	fn path() -> Result<PathBuf> {
		Ok(app_dirs2::app_root(AppDataType::UserConfig, &APP_INFO)?.join("conf.toml"))
	}

	pub(super) fn load() -> Result<Self> {
		let p = Self::path()?;

		if p.is_file() {
			let s = fs::read_to_string(p)?;

			Ok(toml::from_str(&s)?)
		} else {
			Ok(Self::default())
		}
	}

	pub(super) fn save(&self) -> Result<()> {
		let p = Self::path()?;

		fs::write(p, toml::to_string_pretty(self)?)?;

		Ok(())
	}
}
impl Default for RelayerConf {
	fn default() -> Self {
		Self {
			seed: Keypair::new(&Secp256k1::new(), &mut rand::thread_rng())
				.display_secret()
				.to_string(),
			network: Network::Testnet,
		}
	}
}
impl TryFrom<Relayer> for RelayerConf {
	type Error = Error;

	fn try_from(value: Relayer) -> Result<Self> {
		Ok(Self { seed: format!("0x{}", value.keypair.display_secret()), network: value.network })
	}
}
