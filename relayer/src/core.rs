// crates.io
use bitcoin::{
	blockdata::{
		locktime::absolute::LockTime,
		transaction::{Transaction, Version},
	},
	consensus,
	key::Keypair,
	opcodes::all::OP_RETURN,
	script::PushBytesBuf,
	secp256k1::{Message, Secp256k1},
	sighash::SighashCache,
	Address, Amount, EcdsaSighashType, Network, PublicKey, Script, TxIn, TxOut,
};
// self
use crate::{prelude::*, relayer::FeeConf};

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
	pub async fn build(self) -> Result<String>
	where
		E: AsRef<[u8]>,
	{
		let Self { amount, fee_conf, sender, network, recipient, x_target } = self;
		let sender_addr = Address::p2pkh(PublicKey::new(sender.public_key()), network);
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
			let fee = util::estimate_tx_size(input_count, 3) * fee_rate;
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
					.collect();
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

		let script_pubkey = sender_addr.script_pubkey();
		let secp = Secp256k1::new();
		let sender_sk = sender.secret_key();
		let sender_pk = sender.public_key();
		let mut tx =
			Transaction { version: Version::TWO, lock_time: LockTime::ZERO, input, output };

		for i in 0..utxos.len() {
			let sig_hash = SighashCache::new(&tx)
				.legacy_signature_hash(i, &script_pubkey, EcdsaSighashType::All as _)
				.unwrap();
			let msg = Message::from_digest_slice(sig_hash.as_ref())?;
			let sig = secp.sign_ecdsa(&msg, &sender_sk);
			let mut sig_script = sig.serialize_der().to_vec();

			sig_script.push(EcdsaSighashType::All as _);

			let script_sig = Script::builder()
				.push_slice(PushBytesBuf::try_from(sig_script).unwrap())
				.push_slice(sender_pk.serialize())
				.into_script();

			tx.input[i].script_sig = script_sig;
		}

		tracing::debug!("{tx:?}");

		let tx = array_bytes::bytes2hex("", consensus::serialize(&tx));

		tracing::info!("xtx hex: {tx}");

		Ok(tx)
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
		const MAX_ENTITY_SIZE: usize = 80 - 4;

		let XTarget { id, entity } = self;
		let entity = entity.as_ref();
		let mut mark = [0; 80];

		mark[..4].copy_from_slice(&id.to_le_bytes());

		if entity.len() > MAX_ENTITY_SIZE {
			Err(Error::EntityTooLarge { max: MAX_ENTITY_SIZE, actual: entity.len() })?;
		}

		mark[4..4 + entity.len()].copy_from_slice(entity);

		let mut buf = PushBytesBuf::new();

		// This is safe because the length of mark is always 80.
		buf.extend_from_slice(&mark).unwrap();

		Ok(buf)
	}
}
