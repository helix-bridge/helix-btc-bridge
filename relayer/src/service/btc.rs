// std
use std::sync::Arc;
// crates.io
use bitcoin::Network;
use reqwest::ClientBuilder;
use tokio::runtime::Runtime;
// self
use super::Relay;
use crate::{
	chain::btc::{api::mempool::Api, *},
	conf::btc::*,
	http::Client,
	prelude::*,
	x::*,
};

#[derive(Debug)]
pub(super) struct Relayer {
	api: Api<Client>,
	network: Network,
	vault: TaprootKey,
	fee_conf: FeeConf,
}
impl Relayer {
	const NAME: &'static str = "helix-btc-relayer";

	// For testing.
	#[allow(unused)]
	async fn transfer(&self) -> Result<()> {
		let Relayer { api, network, vault, fee_conf } = self;
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

	async fn watch(&self) -> Result<()> {
		let txs = self.api.get_addr_txs_chain(&self.vault.address, None::<&str>).await?;

		Ok(())
	}
}
impl TryFrom<Conf> for Relayer {
	type Error = Error;

	fn try_from(conf: Conf) -> Result<Self> {
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

		Ok(Self { api, network, vault, fee_conf })
	}
}
impl X for Relayer {
	const ID: Id = Id(0);
}
impl Relay for Relayer {
	fn name(&self) -> &'static str {
		Self::NAME
	}

	fn run(&self, runtime: Arc<Runtime>) -> Result<()> {
		tracing::info!("running {}", self.name());

		let self_static: &'static _ = unsafe { &*(self as *const Self) };

		runtime.block_on(async move {
			self_static.watch().await?;

			// loop {
			// }

			Ok::<(), Error>(())
		})?;

		Ok(())
	}
}
