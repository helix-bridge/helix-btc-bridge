#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("[api] max retries exceeded after {retries} attempts")]
	ExceededMaxRetries { retries: u32 },
}
