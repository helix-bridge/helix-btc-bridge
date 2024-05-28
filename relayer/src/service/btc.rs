mod util;

// std
use std::{sync::Arc, time::Duration};
// crates.io
use bitcoin::Network;
use chrono::Utc;
use deadpool_sqlite::Pool;
use reqwest::ClientBuilder;
use tokio::{task, time};
// self
use super::{Context, Relay};
use crate::{
	chain::btc::{api::mempool::Api, *},
	conf::btc::*,
	http::Client,
	prelude::*,
	sql::*,
	x::*,
};

#[derive(Debug)]
pub(super) struct Relayer {
	context: Context,
	api: Api<Client>,
	network: Network,
	vault: TaprootKey,
	fee_conf: FeeConf,
}
impl Relayer {
	pub fn new(conf: Conf, context: Context) -> Result<Self> {
		let Conf { network, vault_secret_key, fee_conf } = conf;
		let api = Api {
			http: Client(ClientBuilder::new().user_agent(Self::NAME).build()?),
			base_uri: if matches!(network, Network::Testnet) {
				"https://mempool.space/testnet/api"
			} else {
				"https://mempool.space/api"
			},
		};
		let vault = TaprootKey::from_untweaked_keypair(
			vault_secret_key.trim_start_matches("0x").parse()?,
			network,
		);

		Ok(Self { context, api, network, vault, fee_conf })
	}

	// TODO
	#[allow(unused)]
	async fn transfer(&self) -> Result<()> {
		let fee_rate =
			self.api.get_recommended_fee().await?.of(self.fee_conf.strategy) + self.fee_conf.extra;

		tracing::info!("fee rate: {fee_rate}");

		let utxos = self
			.api
			.get_utxos(&self.vault.address)
			.await?
			.into_iter()
			.map(TryInto::try_into)
			.collect::<Result<Vec<_>>>()?;
		let tx_hex = XTxBuilder {
			network: self.network,
			fee_rate,
			sender: &self.vault,
			utxos: utxos.as_slice(),
			recipient: "tb1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247shsh6th",
			x_target: XTarget { id: 0_u32.into(), entity: [b'x'; 32].into() },
			amount: 1,
		}
		.build()?;

		self.api.broadcast(tx_hex).await?;

		Ok(())
	}

	async fn track(&self) -> Result<()> {
		let (bn, txid) =
			self.get_latest().await?.map(|xr| (xr.block_height, xr.txid)).unwrap_or_default();
		let mut after = None;
		let mut xrs = Vec::new();

		'outter: loop {
			let txs = self.api.get_addr_txs_chain(&self.vault.address, after.as_ref()).await?;
			let len = txs.len();

			for tx in txs {
				// Reached the latest transaction, no new incoming transactions.
				if tx.txid == txid
				// Already iterated through all unrecorded txs.
					&& tx.status.block_height <= bn as _
				{
					break 'outter;
				}

				let mut vout = None;

				for v in tx.vout {
					if v.scriptpubkey_type == "op_return" {
						vout = Some(v);

						break;
					}
				}

				let Some(vout) = vout else {
					// Not a valid cross-chain tx.
					continue;
				};
				let Ok(xt) = util::extract_xtarget(vout.scriptpubkey_asm) else {
					// Not a valid cross-chain tx.
					continue;
				};

				xrs.push(XRecord {
					block_height: tx.status.block_height as _,
					txid: tx.txid.clone(),
					target: xt.id,
					recipient: array_bytes::bytes2hex("0x", xt.entity.as_bytes()),
					amount: vout.value as _,
					hash: None,
					created_at: Utc::now(),
					finished_at: None,
				});

				tracing::info!("x record found: {}", tx.txid);

				after = Some(tx.txid);
			}

			if len < 25 {
				// No more txs to track.
				break;
			}

			time::sleep(Duration::from_millis(1_000)).await;
		}

		self.insert(xrs).await?;

		Ok(())
	}
}
impl X for Relayer {
	const NAME: &'static str = "btc-x";
}
impl Sql for Relayer {
	async fn pool(&self) -> &Arc<Pool> {
		&self.context.sql
	}
}
impl Relay for Relayer {
	fn name(&self) -> &'static str {
		Self::NAME
	}

	fn init(&self) -> Result<()> {
		task::block_in_place(|| {
			self.context.runtime.block_on(async { <Self as Sql>::init(self).await })
		})
	}

	fn run(&self) -> Result<()> {
		tracing::info!("running {}", self.name());

		let ss = unsafe { &*(self as *const Self) };

		task::block_in_place(|| {
			self.context.runtime.block_on(async move {
				let mut interval = time::interval(Duration::from_millis(5_000));
				// TODO: https://github.com/rust-lang/rust/issues/35121.

				loop {
					tokio::select! {
						// TODO: test only.
						// _ = ss.transfer() => { return Ok(()); }
						_ = interval.tick() => { ss.track().await?; }
					}
				}
			})
		})
	}
}
