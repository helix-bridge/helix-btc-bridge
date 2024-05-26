// crates.io
use bitcoin::script::PushBytesBuf;
// self
use crate::prelude::*;

// TODO?: `Encode` and `Decode` traits.

pub trait X {
	const ID: Id;
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum XEntity {
	Address20([u8; 20]),
	Address32([u8; 32]),
}
impl XEntity {
	const LENGTH_MARKER_SIZE: usize = 1;

	pub fn from_bytes<S>(s: S) -> Result<Self>
	where
		S: AsRef<[u8]>,
	{
		let s = s.as_ref();
		let e = match s.len() {
			20 => Self::Address20(array_bytes::slice2array(s).map_err(Error::ArrayBytes)?),
			32 => Self::Address32(array_bytes::slice2array(s).map_err(Error::ArrayBytes)?),
			_ => Err(XError::EntitySizeInvalid(s.len()))?,
		};

		Ok(e)
	}

	pub fn as_bytes(&self) -> &[u8] {
		match self {
			XEntity::Address20(v) => v,
			XEntity::Address32(v) => v,
		}
	}

	fn encode(&self) -> Vec<u8> {
		let mut v = Vec::new();

		match self {
			XEntity::Address20(a) => {
				v.push(a.len() as _);
				v.extend_from_slice(a);

				v
			},
			XEntity::Address32(a) => {
				v.push(a.len() as _);
				v.extend_from_slice(a);

				v
			},
		}
	}

	fn decode<S>(s: S) -> Result<Self>
	where
		S: AsRef<[u8]>,
	{
		let s = s.as_ref();

		debug_assert_eq!(Self::LENGTH_MARKER_SIZE, 1);

		let e = match s[0] {
			20 => XEntity::Address20(
				array_bytes::slice2array(&s[Self::LENGTH_MARKER_SIZE..])
					.map_err(Error::ArrayBytes)?,
			),
			32 => XEntity::Address32(
				array_bytes::slice2array(&s[Self::LENGTH_MARKER_SIZE..])
					.map_err(Error::ArrayBytes)?,
			),
			_ => Err(XError::EntitySizeInvalid(s[0] as _))?,
		};

		Ok(e)
	}
}
impl From<[u8; 20]> for XEntity {
	fn from(value: [u8; 20]) -> Self {
		Self::Address20(value)
	}
}
impl From<[u8; 32]> for XEntity {
	fn from(value: [u8; 32]) -> Self {
		Self::Address32(value)
	}
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Copy, Debug)]
pub struct Id(pub u32);
impl Id {
	const SIZE: usize = 4;

	pub fn encode(self) -> [u8; Self::SIZE] {
		self.0.to_le_bytes()
	}

	pub fn decode<S>(s: S) -> Result<Self>
	where
		S: AsRef<[u8]>,
	{
		let s = s.as_ref();

		array_bytes::slice_n_into(s).map_err(Error::ArrayBytes)
	}
}
impl From<u8> for Id {
	fn from(value: u8) -> Self {
		Self(value as u32)
	}
}
impl From<u16> for Id {
	fn from(value: u16) -> Self {
		Self(value as u32)
	}
}
impl From<u32> for Id {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
impl From<[u8; 4]> for Id {
	fn from(value: [u8; 4]) -> Self {
		Self(u32::from_le_bytes(value))
	}
}

/// Data structure composition:
///
/// `[0..4](id) ++ [0..1](length) ++ [u8; 75](bytes)`
///
/// 1. `id` (4 bytes):
///    - A 4-byte field for a unique identifier.
/// 2. `length` (1 byte):
///    - A 1-byte field indicating the length of the `bytes` field.
/// 3. `bytes` (up to 75 bytes):
///    - An array of up to 75 bytes for storing data.
///    - The actual length is specified by the `length` field.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct XTarget {
	pub id: Id,
	pub entity: XEntity,
}
impl XTarget {
	pub fn encode(&self) -> Result<PushBytesBuf> {
		let XTarget { id, entity } = self;
		let mut buf = PushBytesBuf::new();

		// These are safe because the total size of `XTarget` is always less than 80.
		buf.extend_from_slice(&id.encode()).unwrap();

		let entity = entity.as_bytes();

		buf.push(entity.len() as u8).unwrap();
		buf.extend_from_slice(entity).unwrap();

		Ok(buf)
	}

	pub fn decode<S>(s: S) -> Result<Self>
	where
		S: AsRef<[u8]>,
	{
		let s = s.as_ref();
		let id = Id::decode(&s[..Id::SIZE])?;
		let entity = XEntity::decode(&s[Id::SIZE..])?;

		Ok(Self { id, entity })
	}
}
#[test]
fn x_target_codec_should_work() {
	[
		(
			Id(2020),
			['x' as u8; 20].as_slice(),
			[
				228, 7, 0, 0, 20, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120,
				120, 120, 120, 120, 120, 120, 120,
			]
			.as_slice(),
		),
		(
			Id(3232),
			&['x' as u8; 32],
			&[
				160, 12, 0, 0, 32, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120,
				120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120,
				120, 120, 120,
			],
		),
	]
	.iter()
	.for_each(|&(id, entity, expected_encoded)| {
		let xt = XTarget { id, entity: XEntity::from_bytes(entity).unwrap() };
		let encoded = xt.encode().unwrap();
		let encoded = encoded.as_bytes();

		assert_eq!(encoded, expected_encoded);

		let decoded = XTarget::decode(encoded).unwrap();

		assert_eq!(xt, decoded);
	});
}
