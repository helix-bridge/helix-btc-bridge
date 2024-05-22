mod chain;
mod conf;
mod error;
mod http;
mod service;
mod x;

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

	Ok(())
}
