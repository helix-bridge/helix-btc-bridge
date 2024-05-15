// std
#[cfg(test)] use std::fmt::{Formatter, Result as FmtResult};
// crates.io
use bitcoin::OutPoint;
// self
use crate::prelude::*;

pub type Satoshi = u64;
#[test]
fn max_btc_in_u64_should_work() {
	let max_u64 = Satoshi::MAX;
	let max_btc = 21_000_000_u64 * 100_000_000;

	assert!(max_u64 > max_btc);
}

pub type Id = u32;
pub type Index = u32;

#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(test), derive(Debug))]
pub struct Utxo {
	pub outpoint: OutPoint,
	pub value: Satoshi,
}
#[cfg(test)]
impl Utxo {
	pub(crate) fn new(value: Satoshi) -> Self {
		Self { value, outpoint: OutPoint { txid: Txid::all_zeros(), vout: 0 } }
	}
}
#[cfg(test)]
impl Debug for Utxo {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Utxo").field("value", &self.value).finish()
	}
}
impl TryFrom<crate::api::mempool::Utxo> for Utxo {
	type Error = Error;

	fn try_from(value: crate::api::mempool::Utxo) -> Result<Self> {
		Ok(Self {
			outpoint: OutPoint {
				txid: value.txid.parse().map_err(BitcoinError::HexToArray)?,
				vout: value.vout,
			},
			value: value.value,
		})
	}
}
