use crate::spec::Spec;
use reqwest::Response;
use serde_json::from_slice;

pub async fn spec(url: &str) -> reqwest::Result<Spec> {
	let res = reqwest::get(url).await?;
	Ok(from_slice(&res.bytes().await?).unwrap())
}

pub async fn get(url: &str) -> reqwest::Result<Response> {
	reqwest::get(url).await
}
