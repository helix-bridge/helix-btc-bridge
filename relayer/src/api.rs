pub mod mempool;

// std
use std::sync::Arc;
// crates.io
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
// self
use crate::prelude::*;

pub static API: Lazy<Arc<Api>> = Lazy::new(|| Arc::new(Api::new()));

pub struct Api {
	http: Client,
}
impl Api {
	pub fn new() -> Self {
		Self { http: ClientBuilder::new().build().unwrap() }
	}
}
