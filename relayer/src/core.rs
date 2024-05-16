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
use crate::{prelude::*, relayer::FeeConf};

pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

#[derive(Debug)]
pub struct XTxBuilder<'a, E> {
	pub amount: Satoshi,
	pub fee_conf: &'a FeeConf,
	pub network: Network,
	pub sender: &'a Keypair,
	pub recipient: &'a str,
	pub x_target: XTarget<E>,
}
impl<E> XTxBuilder<'_, E> {
	const LOCK_TIME: LockTime = LockTime::ZERO;
	const VERSION: Version = Version::TWO;

	pub async fn build(self) -> Result<String>
	where
		E: AsRef<[u8]>,
	{
		let Self { amount, fee_conf, sender, network, recipient, x_target } = self;
		let (sender_pk, _) = sender.x_only_public_key();
		let sender_addr = Address::p2tr(&SECP256K1, sender_pk, None, network);
		let recipient_addr = util::addr_from_str(recipient, network)?;
		let mark = x_target.encode()?;
		let utxos = Api::acquire()
			.get_utxos(sender_addr.to_string())
			.await?
			.into_iter()
			.map(TryInto::try_into)
			.collect::<Result<Vec<_>>>()?;
		let fee_rate = if let Some(f) = fee_conf.force {
			f
		} else {
			Api::acquire().get_recommended_fee().await?.of(fee_conf.strategy) + fee_conf.extra
		};

		tracing::info!("fee rate: {fee_rate}");

		let mut input_count = 1;
		let (utxos, input, output, fee) = loop {
			// Assume there is always a transfer output, a charge output, and a mark output.
			let fee = util::estimate_tx_size(input_count, 2) * fee_rate;
			let spent = amount + fee;
			let (utxos_amount, utxos) =
				util::select_utxos(&utxos, spent).ok_or(Error::InsufficientFunds {
					required: spent,
					available: utxos.iter().map(|u| u.value).sum(),
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
						script_pubkey: sender_addr.script_pubkey(),
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

		tracing::debug!("{unsigned_tx:?}");

		let sighash_type = TapSighashType::AllPlusAnyoneCanPay;
		let sender_spk = ScriptBuf::new_p2tr(&SECP256K1, sender_pk, None);
		let mut hasher = SighashCache::new(unsigned_tx);

		for (i, utxo) in utxos.into_iter().enumerate() {
			let sighash = hasher
				.taproot_key_spend_signature_hash(
					i,
					&Prevouts::One(
						i,
						TxOut {
							script_pubkey: sender_spk.clone(),
							value: Amount::from_sat(utxo.value),
						},
					),
					sighash_type,
				)
				.map_err(BitcoinError::SigHashTapRoot)?;
			let msg = Message::from_digest_slice(sighash.as_ref())?;
			let sig = SECP256K1.sign_schnorr(&msg, &sender.tap_tweak(&SECP256K1, None).to_inner());
			let sig = Signature { signature: sig, sighash_type };

			*hasher.witness_mut(i).unwrap() = Witness::p2tr_key_spend(&sig);
		}

		let tx = hasher.into_transaction();
		let tx_hex = array_bytes::bytes2hex("", consensus::serialize(&tx));

		tracing::info!("xtx hex: {tx_hex}");

		Ok(tx_hex)
	}
}
#[derive(Debug)]
pub struct XTarget<E> {
	pub id: Id,
	pub entity: E,
}
impl<E> XTarget<E> {
	const SIZE: usize = 80;
	const SIZE_ENTITY: usize = Self::SIZE - Self::SIZE_ID;
	const SIZE_ID: usize = 4;

	fn encode(&self) -> Result<PushBytesBuf>
	where
		E: AsRef<[u8]>,
	{
		let XTarget { id, entity } = self;
		let entity = entity.as_ref();
		let mut mark = [0; 80];

		mark[..4].copy_from_slice(&id.encode());

		if entity.len() > Self::SIZE_ENTITY {
			Err(Error::EntityTooLarge { max: Self::SIZE_ENTITY, actual: entity.len() })?;
		}

		mark[4..4 + entity.len()].copy_from_slice(entity);

		let mut buf = PushBytesBuf::new();

		// This is safe because the length of mark is always 80.
		buf.extend_from_slice(&mark).unwrap();

		Ok(buf)
	}
}
impl<'a> XTarget<&'a [u8; 76]> {
	fn decode(s: &'a [u8]) -> Result<Self> {
		let id = Id::decode(&s[..4])?;

		if s[4..].len() != 76 {
			Err(Error::EntityTooLarge { max: 76, actual: s[4..].len() })?;
		}

		let entity = array_bytes::slice2array_ref(&s[4..80]).map_err(Error::ArrayBytes)?;

		Ok(Self { id, entity })
	}
}
