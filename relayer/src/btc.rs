// crates.io
use bitcoin::{
	blockdata::{
		locktime::absolute::LockTime,
		transaction::{Transaction, Version},
	},
	Network, TxIn,
};
// self
use crate::prelude::*;

struct XTxBuilder<'a> {
	network: Network,
	from: &'a str,
	to: &'a str,
	amount: Amount,
	target: ChainId,
}
impl XTxBuilder<'_> {
	async fn build(self) -> Result<Transaction> {
		let XTxBuilder { network, from, to, amount, target } = self;
		let utxos = API
			.get_utxos(from)
			.await?
			.into_iter()
			.map(Utxo::try_from)
			.collect::<Result<Vec<_>>>()?;

		util::select_utxos(&utxos, amount);

		let mut tx = Transaction {
			version: Version::TWO,
			lock_time: LockTime::ZERO,
			input: Vec::new(),
			output: Vec::new(),
		};

		Ok(tx)
	}
}
