//! Helix BTC Relayer

#![deny(
	// clippy::all,
	missing_docs,
	unused_crate_dependencies,
	// warnings,
)]

mod chain;
mod conf;
mod error;
mod http;
mod service;
mod sql;
mod x;

mod prelude {
	pub use crate::error::*;

	pub type Result<T> = std::result::Result<T, Error>;
}

fn main() -> prelude::Result<()> {
	color_eyre::install().unwrap();
	tracing_subscriber::fmt::init();

	service::run()
}
