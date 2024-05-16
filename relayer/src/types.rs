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

pub type Index = u32;

#[derive(Clone, Copy, Debug)]
pub struct Id(pub u32);
impl Id {
	pub fn encode(self) -> [u8; 4] {
		self.0.to_le_bytes()
	}

	pub fn decode<S>(s: S) -> Result<Self>
	where
		S: AsRef<[u8]>,
	{
		let s = s.as_ref();

		array_bytes::slice_n_into(s).map_err(Error::ArrayBytes)
	}
}
impl From<u32> for Id {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
impl From<[u8; 4]> for Id {
	fn from(value: [u8; 4]) -> Self {
		Self(u32::from_le_bytes(value))
	}
}

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
