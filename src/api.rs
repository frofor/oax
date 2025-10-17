use crate::spec::Spec;
use reqwest::blocking::Response;
use serde_json::from_slice;

pub fn fetch_spec_blocking(url: &str) -> reqwest::Result<Spec> {
	let res = reqwest::blocking::get(url)?;
	Ok(from_slice(&res.bytes()?).unwrap())
}

pub fn get_blocking(api_url: &str, path: &str) -> reqwest::Result<Response> {
	reqwest::blocking::get(format!("{api_url}{path}"))
}
