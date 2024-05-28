#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
	#[error("[service] fail to extract {item} from {src}")]
	FailToExtractItem { item: &'static str, src: String },
}
