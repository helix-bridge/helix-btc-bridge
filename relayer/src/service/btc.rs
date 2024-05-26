// std
use std::sync::Arc;
// crates.io
use bitcoin::Network;
use deadpool_sqlite::Pool;
use reqwest::ClientBuilder;
// self
use super::{Context, Relay};
use crate::{
	chain::btc::{api::mempool::Api, *},
	conf::btc::*,
	http::Client,
	prelude::*,
	sql::Sql,
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
	const NAME: &'static str = "helix-btc-relayer";

	pub fn init(conf: Conf, context: Context) -> Result<Self> {
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

	// For testing.
	#[allow(unused)]
	async fn transfer(&self) -> Result<()> {
		let Relayer { context, api, network, vault, fee_conf } = self;
		let fee_rate = api.get_recommended_fee().await?.of(fee_conf.strategy) + fee_conf.extra;

		tracing::info!("fee rate: {fee_rate}");

		let utxos = api
			.get_utxos(&vault.address)
			.await?
			.into_iter()
			.map(TryInto::try_into)
			.collect::<Result<Vec<_>>>()?;
		let tx_hex = XTxBuilder {
			network: *network,
			fee_rate,
			sender: vault,
			utxos: utxos.as_slice(),
			recipient: "tb1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247shsh6th",
			x_target: XTarget { id: 0_u32.into(), entity: [0; 32].into() },
			amount: 1,
		}
		.build()?;

		api.broadcast(tx_hex).await?;

		Ok(())
	}

	// async fn watch(&self) -> Result<()> {
	// 	let txs = self.api.get_addr_txs_chain(&self.vault.address, None::<&str>).await?;

	// 	Ok(())
	// }
}
impl X for Relayer {
	const ID: Id = Id(0);
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

	fn start(&self) -> Result<()> {
		tracing::info!("running {}", self.name());

		let self_static: &'static _ = unsafe { &*(self as *const Self) };

		self.context.runtime.block_on(async move {
			loop {}

			Ok::<(), Error>(())
		})?;

		Ok(())
	}
}
