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
	script::PushBytesBuf,
	secp256k1::{All, Message, Secp256k1},
	sighash::{Prevouts, SighashCache},
	taproot::Signature,
	Address, Amount, Network, Script, ScriptBuf, TapSighashType, TxIn, TxOut, Witness,
};
use once_cell::sync::Lazy;
// self
use crate::prelude::*;

const X_TARGET_SIZE: usize = 80;
const X_TARGET_ID_SIZE: usize = 4;
const X_TARGET_ENTITY_SIZE: usize = X_TARGET_SIZE - X_TARGET_ID_SIZE;

static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

#[derive(Debug)]
pub struct XTxBuilder<'a, E> {
	pub amount: Satoshi,
	pub fee_rate: Satoshi,
	pub network: Network,
	pub sender: &'a TaprootKey,
	pub recipient: &'a str,
	pub utxos: &'a [Utxo],
	pub x_target: XTarget<E>,
}
impl<E> XTxBuilder<'_, E> {
	const LOCK_TIME: LockTime = LockTime::ZERO;
	const VERSION: Version = Version::TWO;

	pub fn build(self) -> Result<String>
	where
		E: AsRef<[u8]>,
	{
		let Self { amount, fee_rate, sender, network, recipient, utxos, x_target } = self;
		let recipient_addr = util::addr_from_str(recipient, network)?;
		let mark = x_target.encode()?;
		let mut input_count = 1;
		let (utxos, input, output, fee) = loop {
			// Assume there is always a transfer output, a charge output, and a mark output.
			let (tx_size, v_size) = util::estimate_tx_size(
				input_count,
				2,
				// OP_RETURN base.
				1 + 83 + Amount::SIZE as Satoshi,
			);

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
					TxOut {
						script_pubkey: Script::builder()
							.push_opcode(OP_RETURN)
							.push_slice(mark)
							.into_script(),
						value: Amount::ZERO,
					},
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
	pub address: String,
	pub keypair: Keypair,
	pub script_public_key: ScriptBuf,
}
impl TaprootKey {
	pub fn from_untweaked_keypair(keypair: Keypair, network: Network) -> Self {
		let (x_only_public_key, _) = keypair.x_only_public_key();
		let address = Address::p2tr(&SECP256K1, x_only_public_key, None, network);
		let keypair = keypair.tap_tweak(&SECP256K1, None).to_inner();
		let script_public_key = address.script_pubkey();

		Self { address: address.to_string(), keypair, script_public_key }
	}
}

#[derive(Debug)]
pub struct XTarget<E> {
	pub id: Id,
	pub entity: E,
}
impl<E> XTarget<E> {
	fn encode(&self) -> Result<PushBytesBuf>
	where
		E: AsRef<[u8]>,
	{
		let XTarget { id, entity } = self;
		let entity = entity.as_ref();
		let mut mark = [0; X_TARGET_SIZE];

		mark[..4].copy_from_slice(&id.encode());

		if entity.len() > X_TARGET_ENTITY_SIZE {
			Err(ChainBtcError::EntityTooLarge { max: X_TARGET_ENTITY_SIZE, actual: entity.len() })?;
		}

		mark[4..4 + entity.len()].copy_from_slice(entity);

		let mut buf = PushBytesBuf::new();

		// This is safe because the length of mark is always 80.
		buf.extend_from_slice(&mark).unwrap();

		Ok(buf)
	}
}
impl<'a> XTarget<&'a [u8; X_TARGET_SIZE]> {
	fn decode(s: &'a [u8]) -> Result<Self> {
		let id = Id::decode(&s[..X_TARGET_ID_SIZE])?;

		if s[X_TARGET_ID_SIZE..].len() != X_TARGET_ENTITY_SIZE {
			Err(ChainBtcError::EntityTooLarge {
				max: X_TARGET_ENTITY_SIZE,
				actual: s[X_TARGET_ID_SIZE..].len(),
			})?;
		}

		let entity = array_bytes::slice2array_ref(&s[X_TARGET_ID_SIZE..X_TARGET_SIZE])
			.map_err(Error::ArrayBytes)?;

		Ok(Self { id, entity })
	}
}