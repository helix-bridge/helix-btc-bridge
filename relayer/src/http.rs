// std
use std::time::Duration;
// crates.io
use bytes::Bytes;
use reqwest::{Body, Client as RClient, IntoUrl};
use serde::de::DeserializeOwned;
use tokio::time;
// self
use crate::prelude::*;

pub trait Http {
	async fn get<U>(&self, uri: U) -> Result<Bytes>
	where
		U: IntoUrl;

	async fn get_with_reties<U>(&self, uri: U, retries: u32, retry_delay_ms: u64) -> Result<Bytes>
	where
		U: IntoUrl,
	{
		let u = uri.as_str();

		for i in 1..=retries {
			match self.get(u).await {
				Ok(r) => return Ok(r),
				Err(e) => {
					tracing::error!(
						"attempt {i}/{retries} failed for {u}: {e:?}, \
							retrying in {retry_delay_ms}ms"
					);
					time::sleep(Duration::from_millis(retry_delay_ms)).await;
				},
			}
		}

		Err(ApiError::ExceededMaxRetries { retries })?
	}

	async fn post<U, B>(&self, uri: U, body: B) -> Result<Bytes>
	where
		U: IntoUrl,
		B: Into<Body>;

	async fn post_with_retries<U, B>(
		&self,
		uri: U,
		body: B,
		retries: u32,
		retry_delay_ms: u64,
	) -> Result<Bytes>
	where
		U: IntoUrl,
		B: Clone + Into<Body>,
	{
		let u = uri.as_str();

		for i in 1..=retries {
			match self.post(u, body.clone()).await {
				Ok(r) => return Ok(r),
				Err(e) => {
					tracing::error!(
						"attempt {i}/{retries} failed for {u}: {e:?}, \
							retrying in {retry_delay_ms}ms"
					);
					time::sleep(Duration::from_millis(retry_delay_ms)).await;
				},
			}
		}

		Err(ApiError::ExceededMaxRetries { retries })?
	}
}

pub trait Response
where
	Self: AsRef<[u8]>,
{
	fn json<D>(&self) -> Result<D>
	where
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

	fn text(&self) -> String {
		String::from_utf8_lossy(self.as_ref()).into()
	}
}
impl Response for Bytes {}

#[derive(Debug)]
pub struct Client(pub RClient);
impl Http for Client {
	async fn get<U>(&self, uri: U) -> Result<Bytes>
	where
		U: IntoUrl,
	{
		Ok(self.0.get(uri).send().await?.bytes().await?)
	}

	async fn post<U, B>(&self, uri: U, body: B) -> Result<Bytes>
	where
		U: IntoUrl,
		B: Into<Body>,
	{
		Ok(self.0.post(uri).body(body).send().await?.bytes().await?)
	}
}
