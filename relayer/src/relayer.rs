mod conf;
use conf::RelayerConf;

// crates.io
use bitcoin::{key::Keypair, secp256k1::Secp256k1, Network};
// self
use crate::prelude::*;

#[derive(Debug)]
pub struct Relayer {
	pub keypair: Keypair,
	pub network: Network,
}
impl Relayer {
	pub fn load() -> Result<Self> {
		RelayerConf::load()?.try_into()
	}

	pub fn save(self) -> Result<()> {
		RelayerConf::try_from(self)?.save()
	}
}
impl TryFrom<RelayerConf> for Relayer {
	type Error = Error;

	fn try_from(value: RelayerConf) -> Result<Self> {
		Ok(Self {
			keypair: Keypair::from_seckey_str(
				&Secp256k1::new(),
				value.seed.trim_start_matches("0x"),
			)?,
			network: value.network,
		})
	}
}
