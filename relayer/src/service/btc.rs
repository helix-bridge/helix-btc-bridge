// crates.io
use bitcoin::Network;
use reqwest::ClientBuilder;
use tokio::runtime::Runtime;
// self
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
	fee_conf: FeeConf,
	key: TaprootKey,
	network: Network,
}
impl Relayer {
	const NAME: &'static str = "helix-btc-relayer";

	// For testing.
	#[allow(unused)]
	async fn transfer(&self) -> Result<()> {
		let Relayer { api, fee_conf, key, network } = self;
		let fee_rate = api.get_recommended_fee().await?.of(fee_conf.strategy) + fee_conf.extra;

		tracing::info!("fee rate: {fee_rate}");

		let utxos = api
			.get_utxos(&key.address)
			.await?
			.into_iter()
			.map(TryInto::try_into)
			.collect::<Result<Vec<_>>>()?;
		let tx_hex = XTxBuilder {
			amount: 1,
			fee_rate,
			sender: key,
			network: *network,
			recipient: "tb1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247shsh6th",
			utxos: utxos.as_slice(),
			x_target: XTarget { id: 0_u32.into(), entity: [0; 32].into() },
		}
		.build()?;

		api.broadcast(tx_hex).await?;

		Ok(())
	}

	// async fn watch(&self) -> Result<()> {}
}
impl TryFrom<Conf> for Relayer {
	type Error = Error;

	fn try_from(conf: Conf) -> Result<Self> {
		let api = Api {
			http: Client(ClientBuilder::new().user_agent(Self::NAME).build()?),
			base_uri: if matches!(conf.network, Network::Testnet) {
				"https://mempool.space/testnet/api"
			} else {
				"https://mempool.space/api"
			},
		};
		let key = TaprootKey::from_untweaked_keypair(
			conf.secret_key.trim_start_matches("0x").parse()?,
			conf.network,
		);

		Ok(Self { api, fee_conf: conf.fee_conf, key, network: conf.network })
	}
}
impl super::Relayer for Relayer {
	fn name(&self) -> &'static str {
		Self::NAME
	}

	fn run(&self) -> Result<()> {
		tracing::info!("running {}", self.name());

		let Relayer { api, fee_conf, key, network }: &'static _ =
			unsafe { &*(self as *const Self) };

		Runtime::new()?.block_on(async move {
			// loop {
			// }
		});

		Ok(())
	}
}
