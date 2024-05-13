// std
use std::str::FromStr;
// crates.io
#[cfg(test)] use bitcoin::hashes::Hash;
use bitcoin::Txid;
// self
use crate::prelude::*;

pub type Amount = u64;
#[test]
fn max_btc_in_u64_should_work() {
	let max_u64 = Amount::MAX;
	let max_btc = 21_000_000_u64 * 100_000_000;

	assert!(max_u64 > max_btc);
}

pub type ChainId = u32;
pub type Index = u32;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct Utxo {
	pub txid: Txid,
	pub value: Amount,
	pub vout: Index,
}
#[cfg(test)]
impl Default for Utxo {
	fn default() -> Self {
		Self { txid: Txid::from_raw_hash(Hash::all_zeros()), value: 0, vout: 0 }
	}
}
impl TryFrom<crate::api::mempool::Utxo> for Utxo {
	type Error = Error;

	fn try_from(value: crate::api::mempool::Utxo) -> Result<Self> {
		Ok(Self { txid: Txid::from_str(&value.txid)?, value: value.value, vout: value.vout })
	}
}
