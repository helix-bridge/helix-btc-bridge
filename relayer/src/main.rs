//! Helix BTC Relayer

// #![deny(clippy::all)]
#![deny(missing_docs)]
// #![deny(unused_crate_dependencies)]
// #![deny(warnings)]

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

const APP_INFO: app_dirs2::AppInfo =
	app_dirs2::AppInfo { name: "helix-btc-bridge-relayer", author: "Xavier Lau" };

fn main() -> prelude::Result<()> {
	color_eyre::install().unwrap();
	tracing_subscriber::fmt::init();
	service::run()?;

	Ok(())
}
