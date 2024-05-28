// self
use crate::{prelude::*, x::XTarget};

pub fn extract_xtarget<S>(s: S) -> Result<XTarget>
where
	S: AsRef<str>,
{
	let s = s.as_ref();
	let (_, s) = s
		.rsplit_once(' ')
		.ok_or(ServiceError::FailToExtractItem { item: "XTarget", src: s.into() })?;
	let s = array_bytes::hex2bytes(s).map_err(Error::ArrayBytes)?;

	XTarget::decode(s)
}
