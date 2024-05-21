mod chain;
mod conf;
mod error;
mod http;
mod service;

mod prelude {
	pub use crate::error::*;

	pub type Result<T> = std::result::Result<T, Error>;
}
use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install().unwrap();
	tracing_subscriber::fmt::init();
	service::run()?;

	// let c = relayer::Relayer::load()?;
	// let tx_hex = core::XTxBuilder {
	// 	amount: 1,
	// 	fee_conf: &c.fee_conf,
	// 	sender: &c.keypair,
	// 	network: c.network,
	// 	recipient: "tb1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247shsh6th",
	// 	x_target: core::XTarget { id: 0.into(), entity: [0; 32] },
	// }
	// .build()
	// .await?;

	// Api::acquire().broadcast(tx_hex).await?;

	Ok(())
}
