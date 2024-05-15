pub mod mempool;
pub mod node;

// std
use std::{sync::Arc, time::Duration};
// crates.io
use once_cell::sync::OnceCell;
use reqwest::{Body, Client, ClientBuilder};
use serde::de::DeserializeOwned;
use tokio::time;
// self
use crate::prelude::*;

pub static API: OnceCell<Arc<Api>> = OnceCell::new();

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

	pub async fn get<D>(&self, uri: &str) -> Result<D>
	where
		D: DeserializeOwned,
	{
		let b = self.http.get(&format!("{}/{uri}", self.base_uri)).send().await?.bytes().await?;

		match serde_json::from_slice(&b) {
			Ok(d) => Ok(d),
			Err(e) => {
				tracing::error!("{}", String::from_utf8_lossy(&b));

				Err(e)?
			},
		}
	}

	pub async fn get_with_reties<D>(
		&self,
		uri: &str,
		retries: u32,
		retry_delay_ms: u64,
	) -> Result<D>
	where
		D: DeserializeOwned,
	{
		for i in 1..=retries {
			match self.get::<D>(uri).await {
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

	pub async fn post<B, D>(&self, uri: &str, body: B) -> Result<D>
	where
		B: Into<Body>,
		D: DeserializeOwned,
	{
		let b = self
			.http
			.post(&format!("{}/{uri}", self.base_uri))
			.body(body)
			.send()
			.await?
			.bytes()
			.await?;

		match serde_json::from_slice(&b) {
			Ok(d) => Ok(d),
			Err(e) => {
				tracing::error!("{}", String::from_utf8_lossy(&b));

				Err(e)?
			},
		}
	}

	pub async fn post_with_retries<B, D>(
		&self,
		uri: &str,
		body: B,
		retries: u32,
		retry_delay_ms: u64,
	) -> Result<D>
	where
		B: Clone + Into<Body>,
		D: DeserializeOwned,
	{
		for i in 1..=retries {
			match self.post::<_, D>(uri, body.clone()).await {
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
