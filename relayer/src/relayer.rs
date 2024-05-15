mod conf;
pub use conf::{FeeConf, FeeStrategy};

// crates.io
use bitcoin::{key::Keypair, Network};
// self
use crate::prelude::*;

#[derive(Debug)]
pub struct Relayer {
	pub fee_conf: FeeConf,
	pub keypair: Keypair,
	pub network: Network,
}
