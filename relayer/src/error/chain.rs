#[derive(Debug, thiserror::Error)]
pub enum ChainError {
	#[error("[chain] insufficient funds: required {required}, available {available}")]
	InsufficientFunds { required: u128, available: u128 },
}

#[derive(Debug, thiserror::Error)]
pub enum ChainBtcError {
	#[error("[chain::btc] entity size too large: maximum allowed {max}, actual size {actual}")]
	EntityTooLarge { max: usize, actual: usize },
}
