#[derive(Debug, thiserror::Error)]
pub enum ChainError {
	#[error("[chain] insufficient funds: required {required}, available {available}")]
	InsufficientFunds { required: u128, available: u128 },
}
