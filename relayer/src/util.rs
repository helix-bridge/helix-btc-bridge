// std
use std::array;
// self
use crate::{prelude::*, types::Utxo};

// Among various UTXO selection strategies such as First-In-First-Out (FIFO), Largest-First, Best
// Fit, Minimum Subset Sum, and Random Selection, the "Minimum Subset Sum" strategy is chosen for
// its specific advantages in handling small-value transactions. This strategy focuses on
// efficiently utilizing smaller UTXOs, which is particularly beneficial for our primary business of
// facilitating small transactions for users. By prioritizing the use of smaller UTXOs, it helps in
// reducing the wallet's fragmentation and enhances the management of UTXO sets. This method not
// only optimizes the transaction process by minimizing the input count and size but also improves
// user satisfaction by effectively managing their resources.
pub fn select_utxos(utxos: &[Utxo], target: Amount) -> Option<Vec<&Utxo>> {
	let n = utxos.len();
	let max = utxos.iter().map(|u| u.value).sum::<Amount>();
	let mut dp = vec![vec![false; (max + 1) as usize]; n + 1];

	dp[0][0] = true;

	for i in 1..=n {
		let value = utxos[i - 1].value as usize;

		for j in 0..=(max as usize) {
			dp[i][j] = dp[i - 1][j];

			if j >= value {
				dp[i][j] = dp[i][j] || dp[i - 1][j - value];
			}
		}
	}

	let mut possible_target = None;

	for j in target as usize..=(max as usize) {
		if dp[n][j] {
			possible_target = Some(j);

			break;
		}
	}

	if let Some(target) = possible_target {
		let mut solution = Vec::new();
		let mut amount = target;
		let mut i = n;

		while amount > 0 && i > 0 {
			if !dp[i - 1][amount] {
				solution.push(&utxos[i - 1]);
				amount -= utxos[i - 1].value as usize;
			}

			i -= 1;
		}

		solution.reverse();

		return Some(solution);
	}

	None
}
#[test]
fn select_utxos_should_work() {
	let utxos =
		vec![Utxo { value: 1, ..Default::default() }, Utxo { value: 2, ..Default::default() }];
	assert!(select_utxos(&utxos, 4).is_none());

	let utxos =
		vec![Utxo { value: 1, ..Default::default() }, Utxo { value: 2, ..Default::default() }];
	assert_eq!(select_utxos(&utxos, 3).unwrap(), [&utxos[0], &utxos[1]]);

	let utxos = vec![
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 2, ..Default::default() },
		Utxo { value: 3, ..Default::default() },
	];
	assert_eq!(select_utxos(&utxos, 3).unwrap(), [&utxos[0], &utxos[1]]);

	let utxos = vec![
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 2, ..Default::default() },
		Utxo { value: 3, ..Default::default() },
	];
	assert_eq!(select_utxos(&utxos, 4).unwrap(), [&utxos[0], &utxos[2]]);

	let utxos = vec![
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 2, ..Default::default() },
		Utxo { value: 4, ..Default::default() },
	];
	assert_eq!(select_utxos(&utxos, 4).unwrap(), [&utxos[0], &utxos[1], &utxos[2]]);

	let utxos = vec![
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 2, ..Default::default() },
		Utxo { value: 3, ..Default::default() },
		Utxo { value: 4, ..Default::default() },
	];
	assert_eq!(select_utxos(&utxos, 5).unwrap(), [&utxos[1], &utxos[2]]);

	let utxos = vec![
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 2, ..Default::default() },
		Utxo { value: 3, ..Default::default() },
		Utxo { value: 8, ..Default::default() },
	];
	assert_eq!(select_utxos(&utxos, 7).unwrap(), [&utxos[3]]);

	let utxos = vec![
		Utxo { value: 1, ..Default::default() },
		Utxo { value: 2, ..Default::default() },
		Utxo { value: 3, ..Default::default() },
		Utxo { value: 8, ..Default::default() },
		Utxo { value: 9, ..Default::default() },
	];
	assert_eq!(select_utxos(&utxos, 7).unwrap(), [&utxos[3]]);
}
