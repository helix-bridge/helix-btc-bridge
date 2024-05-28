pub mod api;

pub mod types;
use types::*;

mod util;

// crates.io
use bitcoin::{
	blockdata::{
		locktime::absolute::LockTime,
		transaction::{Transaction, Version},
	},
	consensus,
	key::{Keypair, TapTweak},
	opcodes::all::OP_RETURN,
	secp256k1::{All, Message, Secp256k1},
	sighash::{Prevouts, SighashCache},
	taproot::Signature,
	Address, Amount, Network, Script, ScriptBuf, TapSighashType, TxIn, TxOut, Witness,
};
use once_cell::sync::Lazy;
// self
use crate::{prelude::*, x::*};

static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

#[derive(Debug)]
pub struct XTxBuilder<'a> {
	pub network: Network,
	pub fee_rate: Satoshi,
	pub sender: &'a TaprootKey,
	pub utxos: &'a [Utxo],
	pub recipient: &'a str,
	pub x_target: XTarget,
	pub amount: Satoshi,
}
impl XTxBuilder<'_> {
	const LOCK_TIME: LockTime = LockTime::ZERO;
	const VERSION: Version = Version::TWO;

	pub fn build(self) -> Result<String> {
		let Self { network, fee_rate, sender, utxos, recipient, x_target, amount } = self;
		let recipient_addr = util::addr_from_str(recipient, network)?;
		let op_return = TxOut {
			script_pubkey: Script::builder()
				.push_opcode(OP_RETURN)
				.push_slice(x_target.encode()?)
				.into_script(),
			value: Amount::ZERO,
		};
		let mut input_count = 1;
		let (utxos, input, output, fee) = loop {
			// Assume there is always a transfer output, a charge output, and a mark output.
			let (tx_size, v_size) = util::estimate_tx_size(input_count, 2, op_return.size());

			tracing::info!("estimated tx size: {tx_size}");
			tracing::info!("estimated tx virtual size: {v_size}");

			let fee = (v_size.ceil() as Satoshi) * fee_rate;
			let spent = amount + fee;
			let (utxos_amount, utxos) =
				util::select_utxos(utxos, spent).ok_or(ChainError::InsufficientFunds {
					required: spent as _,
					available: utxos.iter().map(|u| u.value).sum::<Satoshi>() as _,
				})?;

			if utxos.len() as Satoshi == input_count {
				let charge = utxos_amount - spent;
				let input = utxos
					.iter()
					.map(|u| TxIn { previous_output: u.outpoint, ..Default::default() })
					.collect::<Vec<_>>();
				let mut output = vec![
					TxOut {
						script_pubkey: recipient_addr.script_pubkey(),
						value: Amount::from_sat(amount),
					},
					op_return,
				];

				if charge != 0 {
					output.push(TxOut {
						script_pubkey: sender.script_public_key.clone(),
						value: Amount::from_sat(charge),
					});
				};

				break (utxos, input, output, fee);
			} else {
				input_count = utxos.len() as _;
			}
		};

		tracing::info!("fee: {fee}");

		let unsigned_tx = Transaction {
			version: Self::VERSION,
			lock_time: Self::LOCK_TIME,
			input,
			output: output.clone(),
		};

		let sighash_type = TapSighashType::AllPlusAnyoneCanPay;
		let mut hasher = SighashCache::new(unsigned_tx);

		for (i, utxo) in utxos.into_iter().enumerate() {
			let sighash = hasher
				.taproot_key_spend_signature_hash(
					i,
					&Prevouts::One(
						i,
						TxOut {
							script_pubkey: sender.script_public_key.clone(),
							value: Amount::from_sat(utxo.value),
						},
					),
					sighash_type,
				)
				.map_err(BitcoinError::SigHashTapRoot)?;
			let msg = Message::from_digest_slice(sighash.as_ref())?;
			let sig = SECP256K1.sign_schnorr(&msg, &sender.keypair);
			let sig = Signature { signature: sig, sighash_type };

			*hasher.witness_mut(i).unwrap() = Witness::p2tr_key_spend(&sig);
		}

		let tx = hasher.into_transaction();

		tracing::debug!("xtx: {tx:?}");

		// dbg!(tx.total_size());
		// tx.input.iter().for_each(|i| {
		// 	dbg!(i.witness.size(), i.base_size());
		// });
		// tx.output.iter().for_each(|o| {
		// 	dbg!(o.size());
		// });

		let tx_hex = array_bytes::bytes2hex("", consensus::serialize(&tx));

		tracing::info!("xtx hex: {tx_hex}");

		Ok(tx_hex)
	}
}

#[derive(Debug)]
pub struct TaprootKey {
	pub keypair: Keypair,
	pub script_public_key: ScriptBuf,
	pub address: String,
}
impl TaprootKey {
	pub fn from_untweaked_keypair(keypair: Keypair, network: Network) -> Self {
		let (x_only_public_key, _) = keypair.x_only_public_key();
		let address = Address::p2tr(&SECP256K1, x_only_public_key, None, network);
		let keypair = keypair.tap_tweak(&SECP256K1, None).to_inner();
		let script_public_key = address.script_pubkey();

		Self { keypair, script_public_key, address: address.to_string() }
	}
}
