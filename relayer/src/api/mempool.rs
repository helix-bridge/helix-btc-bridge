// crates.io
use serde::Deserialize;
// self
use super::*;

#[derive(Debug, Deserialize)]
pub struct Utxo {
	pub status: Status,
	pub txid: String,
	pub value: Amount,
	pub vout: Index,
}
#[derive(Debug, Deserialize)]
pub struct Status {
	pub block_hash: String,
	pub block_height: u32,
	pub block_time: u64,
	pub confirmed: bool,
}

impl Api {
	pub async fn get_utxos(&self, address: &str) -> Result<Vec<Utxo>> {
		let utxos = self
			.http
			.get(&format!("https://mempool.space/testnet/api/address/{address}/utxo"))
			.send()
			.await?
			.json::<Vec<Utxo>>()
			.await?;

		tracing::debug!("{utxos:?}");

		Ok(utxos)
	}

	// pub async fn get_txs(&self, address: &str) -> Result<()> {
	// 	let resp = self
	// 		.http
	// 		.get(&format!("https://mempool.space/testnet/api/address/{address}/txs/chain"))
	// 		.send()
	// 		.await?
	// 		.json::<serde_json::Value>()
	// 		.await?;

	// 	dbg!(resp);

	// 	Ok(())
	// }
}
