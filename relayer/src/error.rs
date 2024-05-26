pub mod api;
pub use api::*;

pub mod chain;
pub use chain::*;

pub mod x;
pub use x::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0:?}")]
	Any(Box<dyn 'static + std::any::Any + Send>),
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	AppDirs2(#[from] app_dirs2::AppDirsError),
	#[error("{0:?}")]
	ArrayBytes(array_bytes::Error),
	#[error(transparent)]
	Bitcoin(#[from] BitcoinError),
	#[error(transparent)]
	DeadpoolSqlite(#[from] DeadpoolSqliteError),
	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),
	#[error(transparent)]
	Rusqlite(#[from] rusqlite::Error),
	#[error(transparent)]
	Secp256k1(#[from] bitcoin::secp256k1::Error),
	#[error(transparent)]
	SerdeJson(#[from] serde_json::Error),
	#[error(transparent)]
	Toml(#[from] toml::de::Error),

	#[error(transparent)]
	Api(#[from] ApiError),
	#[error(transparent)]
	Chain(#[from] ChainError),
	#[error(transparent)]
	X(#[from] XError),
}

#[derive(Debug, thiserror::Error)]
pub enum BitcoinError {
	#[error(transparent)]
	HexToArray(#[from] bitcoin::hex::HexToArrayError),
	#[error(transparent)]
	Parse(#[from] bitcoin::address::ParseError),
	#[error(transparent)]
	SigHashTapRoot(#[from] bitcoin::sighash::TaprootError),
}

#[derive(Debug, thiserror::Error)]
pub enum DeadpoolSqliteError {
	#[error(transparent)]
	Create(#[from] deadpool_sqlite::CreatePoolError),
	#[error(transparent)]
	Interact(#[from] deadpool_sqlite::InteractError),
	#[error(transparent)]
	Pool(#[from] deadpool_sqlite::PoolError),
}
