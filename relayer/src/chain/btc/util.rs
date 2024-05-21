// std
use std::collections::HashMap;
// crates.io
use bitcoin::{address::NetworkUnchecked, Address, Amount, Network, VarInt};
// self
use super::*;
use crate::prelude::*;

pub fn addr_from_str(s: &str, network: Network) -> Result<Address> {
	Ok(s.parse::<Address<NetworkUnchecked>>()
		.map_err(BitcoinError::Parse)?
		.require_network(network)
		.map_err(BitcoinError::Parse)?)
}

// Among various UTXO selection strategies such as First-In-First-Out (FIFO), Largest-First, Best
// Fit, Minimum Subset Sum, and Random Selection, the "Minimum Subset Sum" strategy is chosen for
// its specific advantages in handling small-value transactions. This strategy focuses on
// efficiently utilizing smaller UTXOs, which is particularly beneficial for our primary business of
// facilitating small transactions for users. By prioritizing the use of smaller UTXOs, it helps in
// reducing the wallet's fragmentation and enhances the management of UTXO sets. This method not
// only optimizes the transaction process by minimizing the input count and size but also improves
// user satisfaction by effectively managing their resources.
pub fn select_utxos(utxos: &[Utxo], target: Satoshi) -> Option<(Satoshi, Vec<&Utxo>)> {
	let mut dp = <HashMap<Satoshi, Vec<&Utxo>>>::new();

	dp.insert(0, Vec::new());

	for utxo in utxos {
		let mut combs = <HashMap<Satoshi, Vec<&Utxo>>>::new();

		for (&total, comb) in dp.iter() {
			let new_total = total + utxo.value;

			if !combs.contains_key(&new_total) || combs[&new_total].len() < comb.len() + 1 {
				let mut new_comb = comb.to_owned();

				new_comb.push(utxo);
				combs.insert(new_total, new_comb);
			}
		}
		for (k, v) in combs {
			if !dp.contains_key(&k) || dp[&k].len() < v.len() {
				dp.insert(k, v);
			}
		}

		// dbg!("DP state after processing UTXO={utxo:?}: {dp:?}");
	}

	// Find the exact match or nearest bigger value.
	if let Some(comb) = dp.get(&target) {
		// dbg!("exact match found for target {target}: {comb:?}");

		return Some((target, comb.to_owned()));
	} else {
		let mut min_excess = None;
		let mut best_comb = None;

		for (&total, comb) in dp.iter() {
			if total > target {
				match min_excess {
					Some(prev_total) if total < prev_total => {
						min_excess = Some(total);
						best_comb = Some(comb.to_owned());
					},
					None => {
						min_excess = Some(total);
						best_comb = Some(comb.to_owned());
					},
					_ => {},
				}
			}
		}

		if let Some(best_total) = min_excess {
			return best_comb.map(|c| (best_total, c));
		}
	}

	None
}
#[test]
fn select_utxos_should_work() {
	let utxos = vec![Utxo::new(1), Utxo::new(2)];
	assert!(select_utxos(&utxos, 4).is_none());

	let utxos = vec![Utxo::new(1), Utxo::new(2)];
	assert_eq!(select_utxos(&utxos, 3).unwrap(), [&utxos[0], &utxos[1]]);

	let utxos = vec![Utxo::new(1), Utxo::new(2), Utxo::new(3)];
	assert_eq!(select_utxos(&utxos, 3).unwrap(), [&utxos[0], &utxos[1]]);

	let utxos = vec![Utxo::new(1), Utxo::new(1), Utxo::new(2), Utxo::new(4)];
	assert_eq!(select_utxos(&utxos, 4).unwrap(), [&utxos[0], &utxos[1], &utxos[2]]);

	let utxos = vec![Utxo::new(1), Utxo::new(2), Utxo::new(3), Utxo::new(4)];
	assert_eq!(select_utxos(&utxos, 5).unwrap(), [&utxos[1], &utxos[2]]);

	let utxos = vec![Utxo::new(1), Utxo::new(2), Utxo::new(3), Utxo::new(9)];
	assert_eq!(select_utxos(&utxos, 7).unwrap(), [&utxos[3]]);
}

pub fn estimate_tx_size(
	input_count: Satoshi,
	output_utxo_count: Satoshi,
	extra: Satoshi,
) -> (Satoshi, f64) {
	let base =
		// Version.
		4
		// Lock time.
		+ 4
		// SegWit flag.
		// 2
		// Input count.
		+ VarInt::from(input_count).size() as Satoshi
		// Inputs.
		+ input_count
			* (
				// Base.
				32 + 4 + 1 + 4
				// Witness.
				// + 64 + 1 + 1 + 1
			)
		// Output UTXO count.
		+ VarInt::from(output_utxo_count).size() as Satoshi
		// Output UTXOs.
		+ output_utxo_count
			// Base.
			* (1 + 34 + Amount::SIZE as Satoshi)
		// Extra.
		+ extra;
	let sigwit =
		// SegWit flag.
		2
		// Inputs.
		+ input_count
			// Witness.
			* (64 + 1 + 1 + 1);
	let size = base + sigwit;
	let v_size = (base * 3 + size) as f64 / 4.;

	(size, v_size)
}
