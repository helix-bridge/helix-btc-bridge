#[derive(Debug, thiserror::Error)]
pub enum XError {
	#[error("[x::Entity] invalid size {0}")]
	EntitySizeInvalid(usize),
	#[error("[x::XTarget] invalid bytes {0:?}")]
	XTargetBytesInvalid(Vec<u8>),
}
