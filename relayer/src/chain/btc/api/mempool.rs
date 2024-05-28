//! Rust bindings for the [Mempool API](https://mempool.space/testnet/docs/api/rest).

// crates.io
use serde::{Deserialize, Serialize};
// self
use super::super::types::*;
use crate::{http::*, prelude::*};

#[derive(Debug)]
pub struct Api<H>
where
	H: Http,
{
	pub http: H,
	pub base_uri: &'static str,
}
impl<H> Api<H>
where
	H: Http,
{
	// Get confirmed transaction history for the specified address/scripthash, sorted with newest
	// first. Returns 25 transactions per page. More can be requested by specifying the last `txid`
	// seen by the previous query.
	pub async fn get_addr_txs_chain<S, S1>(&self, address: S, after: Option<S1>) -> Result<Vec<Tx>>
	where
		S: AsRef<str>,
		S1: AsRef<str>,
	{
		let txs = self
			.http
			.get_with_reties(
				format!(
					"{}/address/{}/txs/chain{}",
					self.base_uri,
					address.as_ref(),
					if let Some(a) = after { format!("/{}", a.as_ref()) } else { "".into() }
				),
				3,
				50,
			)
			.await?
			.json::<Vec<Tx>>()?;

		tracing::debug!("{txs:?}");

		Ok(txs)
	}

	// Get the list of unspent transaction outputs associated with the address/scripthash. Available
	// fields: `txid`, `vout`, `value`, and `status` (with the status of the funding tx).
	pub async fn get_utxos<S>(&self, address: S) -> Result<Vec<Utxo>>
	where
		S: AsRef<str>,
	{
		let utxos = self
			.http
			.get_with_reties(format!("{}/address/{}/utxo", self.base_uri, address.as_ref()), 3, 1_000)
			.await?
			.json::<Vec<Utxo>>()?;

		tracing::debug!("get_utxos\n{utxos:?}");

		Ok(utxos)
	}

	// TODO
	#[allow(unused)]
	pub async fn get_utxos_confirmed<S>(&self, address: S) -> Result<Vec<Utxo>>
	where
		S: AsRef<str>,
	{
		let utxos =
			self.get_utxos(address).await?.into_iter().filter(|u| u.status.confirmed).collect();

		tracing::debug!("get_utxos_confirmed\n{utxos:?}");

		Ok(utxos)
	}

	// Returns our currently suggested fees for new transactions.
	pub async fn get_recommended_fee(&self) -> Result<Fees> {
		let fees = self
			.http
			.get_with_reties(format!("{}/v1/fees/recommended", self.base_uri), 3, 1_000)
			.await?
			.json::<Fees>()?;

		tracing::debug!("get_fees\n{fees:?}");

		Ok(fees)
	}

	// Broadcast a raw transaction to the network. The transaction should be provided as hex in the
	// request body. The `txid` will be returned on success.
	pub async fn broadcast<S>(&self, tx_hex: S) -> Result<String>
	where
		S: Into<String>,
	{
		Ok(self
			.http
			.post_with_retries(format!("{}/tx", self.base_uri), tx_hex.into(), 3, 50)
			.await?
			.text())
	}
}

#[derive(Debug, Deserialize)]
pub struct Tx {
	pub txid: String,
	// pub version: u8,
	// pub locktime: BlockNumber,
	// pub vin: Vec<Vin>,
	pub vout: Vec<Vout>,
	// pub size: u32,
	// pub weight: u32,
	// pub sigops: u32,
	// pub fee: Satoshi,
	pub status: Status,
}
// #[derive(Debug, Deserialize)]
// pub struct Vin {
// 	pub txid: String,
// 	pub vout: Index,
// 	pub prevout: Vout,
// 	pub scriptsig: String,
// 	pub scriptsig_asm: String,
// 	pub witness: Vec<String>,
// 	pub is_coinbase: bool,
// 	pub sequence: u32,
// }
#[derive(Debug, Deserialize)]
pub struct Vout {
	// pub scriptpubkey: String,
	pub scriptpubkey_asm: String,
	// TODO: use enum type.
	pub scriptpubkey_type: String,
	pub scriptpubkey_address: Option<String>,
	pub value: Satoshi,
}

#[derive(Debug, Deserialize)]
pub struct Utxo {
	pub txid: String,
	pub vout: Index,
	pub status: Status,
	pub value: Satoshi,
}
#[derive(Debug, Deserialize)]
pub struct Status {
	pub confirmed: bool,
	pub block_height: BlockNumber,
	// 	pub block_hash: String,
	// 	pub block_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
	pub fastest_fee: Satoshi,
	pub half_hour_fee: Satoshi,
	pub hour_fee: Satoshi,
	pub economy_fee: Satoshi,
	pub minimum_fee: Satoshi,
}
impl Fees {
	pub fn of(&self, strategy: FeeType) -> Satoshi {
		match strategy {
			FeeType::Fastest => self.fastest_fee,
			FeeType::HalfHour => self.half_hour_fee,
			FeeType::Hour => self.hour_fee,
			FeeType::Economy => self.economy_fee,
			FeeType::Minimum => self.minimum_fee,
		}
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeeType {
	Fastest,
	HalfHour,
	Hour,
	Economy,
	Minimum,
}
impl Default for FeeType {
	fn default() -> Self {
		Self::Fastest
	}
}
