pub mod mempool;
pub mod node;

// std
use std::{sync::Arc, time::Duration};
// crates.io
use bytes::Bytes;
use once_cell::sync::OnceCell;
use reqwest::{Body, Client, ClientBuilder};
use serde::de::DeserializeOwned;
use tokio::time;
// self
use crate::prelude::*;

pub static API: OnceCell<Arc<Api>> = OnceCell::new();

pub trait Response {
	fn json<D>(&self) -> Result<D>
	where
		Self: AsRef<[u8]>,
		D: DeserializeOwned,
	{
		let s = self.as_ref();

		match serde_json::from_slice(s) {
			Ok(d) => Ok(d),
			Err(e) => {
				tracing::error!("{}", String::from_utf8_lossy(s));

				Err(e)?
			},
		}
	}

	fn text(&self) -> String
	where
		Self: AsRef<[u8]>,
	{
		String::from_utf8_lossy(self.as_ref()).into()
	}
}
impl Response for Bytes {}

#[derive(Debug)]
pub struct Api {
	http: Client,
	base_uri: &'static str,
}
impl Api {
	pub fn init(is_testnet: bool) -> Result<()> {
		API.set(Arc::new(Self {
			http: ClientBuilder::new().user_agent("helix-btc-bridge-relayer").build()?,
			base_uri: if is_testnet {
				"https://mempool.space/testnet/api"
			} else {
				"https://mempool.space/api"
			},
		}))
		.unwrap();

		Ok(())
	}

	pub fn acquire() -> &'static Arc<Self> {
		unsafe { API.get_unchecked() }
	}

	pub async fn get(&self, uri: &str) -> Result<Bytes> {
		Ok(self.http.get(&format!("{}/{uri}", self.base_uri)).send().await?.bytes().await?)
	}

	pub async fn get_with_reties(
		&self,
		uri: &str,
		retries: u32,
		retry_delay_ms: u64,
	) -> Result<Bytes> {
		for i in 1..=retries {
			match self.get(uri).await {
				Ok(r) => return Ok(r),
				Err(e) => {
					tracing::error!(
						"attempt {i}/{retries} failed for {uri}: {e:?}, \
						retrying in {retry_delay_ms}ms"
					);
					time::sleep(Duration::from_millis(retry_delay_ms)).await;
				},
			}
		}

		Err(Error::ExceededMaxRetries { retries })
	}

	pub async fn post<B>(&self, uri: &str, body: B) -> Result<Bytes>
	where
		B: Into<Body>,
	{
		Ok(self
			.http
			.post(&format!("{}/{uri}", self.base_uri))
			.body(body)
			.send()
			.await?
			.bytes()
			.await?)
	}

	pub async fn post_with_retries<B>(
		&self,
		uri: &str,
		body: B,
		retries: u32,
		retry_delay_ms: u64,
	) -> Result<Bytes>
	where
		B: Clone + Into<Body>,
	{
		for i in 1..=retries {
			match self.post(uri, body.clone()).await {
				Ok(r) => return Ok(r),
				Err(e) => {
					tracing::error!(
						"attempt {i}/{retries} failed for {uri}: {e:?}, \
						retrying in {retry_delay_ms}ms"
					);
					time::sleep(Duration::from_millis(retry_delay_ms)).await;
				},
			}
		}

		Err(Error::ExceededMaxRetries { retries })
	}
}
