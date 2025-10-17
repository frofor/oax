use crate::api::fetch_spec_blocking;
use crate::api::get_blocking;
use crate::cli::API_URL;
use crate::cli::ENDPOINT;
use crate::cli::PARAMS;
use crate::cli::SPEC_URL;
use crate::spec::ParamKind;
use clap::ArgMatches;
use std::fmt::Write as _;
use std::process::exit;

pub fn get(matches: &ArgMatches, sub_matches: &ArgMatches) {
	let spec_url: &String = matches.get_one(SPEC_URL).expect("Spec URL should be required");
	let spec = match fetch_spec_blocking(spec_url) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("Failed to fetch specification: {e}");
			exit(1);
		}
	};

	let endpoint: &String = sub_matches.get_one(ENDPOINT).expect("Endpoint should be required");
	let mut url = endpoint.clone();

	let method = spec
		.endpoints
		.get(endpoint)
		.expect("Endpoint should be valid")
		.get
		.as_ref()
		.expect("Endpoint should be valid");

	let params: Vec<_> = sub_matches
		.get_many::<String>(PARAMS)
		.unwrap_or_default()
		.map(|p| p.split_once('=').expect("Param should contain name and value"))
		.collect();

	params
		.iter()
		.filter(|(n, _)| {
			let param = method.params.iter().find(|p| &p.name == n).expect("Param should be valid");
			param.kind == ParamKind::Path
		})
		.for_each(|(n, v)| url = url.replace(&format!("{{{n}}}"), v));

	let query_params: Vec<_> = params
		.iter()
		.filter(|(n, _)| {
			let param = method.params.iter().find(|p| &p.name == n).expect("Param should be valid");
			param.kind == ParamKind::Query
		})
		.collect();
	if !query_params.is_empty() {
		let query =
			query_params.iter().map(|(n, v)| format!("{n}={v}")).collect::<Vec<_>>().join("&");
		let _ = write!(&mut url, "?{query}");
	}

	let api_url: &String = matches.get_one(API_URL).expect("API URL should be required");
	let res = match get_blocking(api_url, &url) {
		Ok(r) => r,
		Err(e) => {
			eprintln!("Request failed: {e}");
			exit(1);
		}
	};
	let body = match res.text() {
		Ok(t) => t,
		Err(e) => {
			eprintln!("Failed to decode response: {e}");
			exit(1);
		}
	};
	println!("{body}");
}
