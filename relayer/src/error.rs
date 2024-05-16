// self
use crate::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	AppDirs2(#[from] app_dirs2::AppDirsError),
	#[error("{0:?}")]
	ArrayBytes(array_bytes::Error),
	#[error(transparent)]
	Bitcoin(#[from] BitcoinError),
	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),
	#[error(transparent)]
	Secp256k1(#[from] bitcoin::secp256k1::Error),
	#[error(transparent)]
	SerdeJson(#[from] serde_json::Error),
	#[error(transparent)]
	Toml(#[from] toml::de::Error),

	#[error("[relayer] insufficient funds: required {required}, available {available}")]
	InsufficientFunds { required: Satoshi, available: Satoshi },
	#[error("[relayer] entity size too large: maximum allowed {max}, actual size {actual}")]
	EntityTooLarge { max: usize, actual: usize },
	#[error("[relayer] max retries exceeded after {retries} attempts")]
	ExceededMaxRetries { retries: u32 },
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
