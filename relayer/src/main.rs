mod api;

mod btc;

mod relayer;
use relayer::Relayer;

mod types;

mod util;

mod prelude {
	pub use anyhow::{Error, Result};

	pub(crate) use crate::util;
	pub use crate::{api::API, types::*};
}
use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install().unwrap();
	tracing_subscriber::fmt::init();

	API.get_utxos("tb1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247shsh6th").await?;

	let c = Relayer::load()?;

	dbg!(&c.keypair.x_only_public_key().0);
	c.save()?;

	Ok(())
}
