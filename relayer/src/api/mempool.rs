// crates.io
use serde::Deserialize;
// self
use super::*;
use crate::relayer::FeeStrategy;

#[derive(Debug, Deserialize)]
pub struct Utxo {
	// pub status: Status,
	pub txid: String,
	pub value: Satoshi,
	pub vout: Index,
}
// #[derive(Debug, Deserialize)]
// pub struct Status {
// 	pub block_hash: String,
// 	pub block_height: u32,
// 	pub block_time: u64,
// 	pub confirmed: bool,
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
	pub economy_fee: Satoshi,
	pub fastest_fee: Satoshi,
	pub half_hour_fee: Satoshi,
	pub hour_fee: Satoshi,
	pub minimum_fee: Satoshi,
}
impl Fees {
	pub fn of(&self, strategy: FeeStrategy) -> Satoshi {
		match strategy {
			FeeStrategy::Economy => self.economy_fee,
			FeeStrategy::Fastest => self.fastest_fee,
			FeeStrategy::HalfHour => self.half_hour_fee,
			FeeStrategy::Hour => self.hour_fee,
			FeeStrategy::Minimum => self.minimum_fee,
		}
	}
}

impl Api {
	pub async fn get_utxos<S>(&self, address: S) -> Result<Vec<Utxo>>
	where
		S: AsRef<str>,
	{
		let utxos = self
			.get_with_reties(&format!("address/{}/utxo", address.as_ref()), 3, 50)
			.await?
			.json::<Vec<Utxo>>()?;

		tracing::debug!("{utxos:?}");

		Ok(utxos)
	}

	pub async fn get_recommended_fee(&self) -> Result<Fees> {
		let fees = self.get_with_reties("v1/fees/recommended", 3, 50).await?.json::<Fees>()?;

		tracing::debug!("{fees:?}");

		Ok(fees)
	}

	pub async fn broadcast<S>(&self, tx_hex: S) -> Result<String>
	where
		S: Into<String>,
	{
		Ok(self.post_with_retries("tx", tx_hex.into(), 3, 50).await?.text())
	}
}
