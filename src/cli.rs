use crate::api::fetch_spec_blocking;
use clap::Arg;
use clap::ArgAction;
use clap::Command;
use clap::ValueHint;
use clap_complete::ArgValueCompleter;
use clap_complete::CompletionCandidate;
use std::collections::HashSet;
use std::env::args;
use std::ffi::OsStr;

pub const GET: &str = "get";
const GET_ALIAS: &str = "g";
pub const POST: &str = "post";
const POST_ALIAS: &str = "p";
pub const PUT: &str = "put";
const PUT_ALIAS: &str = "u";
pub const DELETE: &str = "delete";
const DELETE_ALIAS: &str = "d";
pub const OPTIONS: &str = "options";
const OPTIONS_ALIAS: &str = "o";
pub const HEAD: &str = "head";
const HEAD_ALIAS: &str = "h";
pub const PATCH: &str = "patch";
const PATCH_ALIAS: &str = "a";
pub const TRACE: &str = "trace";
const TRACE_ALIAS: &str = "t";
pub const ENDPOINT: &str = "endpoint";
pub const API_URL: &str = "api-url";
const API_URL_LONG: &str = "api-url";
const API_URL_SHORT: char = 'a';
pub const SPEC_URL: &str = "spec-url";
const SPEC_URL_LONG: &str = "spec-url";
const SPEC_URL_SHORT: char = 's';
pub const PARAMS: &str = "params";
const PARAMS_LONG: &str = "param";
const PARAMS_SHORT: char = 'p';

pub fn cmd() -> Command {
	Command::new("oax")
		.author("frofor <18nraywczifc@protonmail.com>")
		.version("0.1.0")
		.about("oax - an OpenAPI execution client")
		.disable_help_subcommand(true)
		.disable_version_flag(true)
		.arg(
			Arg::new("version")
				.short('v')
				.long("version")
				.help("Print version")
				.action(ArgAction::Version),
		)
		.arg(
			Arg::new(API_URL)
				.short(API_URL_SHORT)
				.long(API_URL_LONG)
				.help("Set URL to API server")
				.required(true)
				.value_name("url")
				.value_hint(ValueHint::Url),
		)
		.arg(
			Arg::new(SPEC_URL)
				.short(SPEC_URL_SHORT)
				.long(SPEC_URL_LONG)
				.help("Set URL to specification")
				.required(true)
				.value_name("url")
				.value_hint(ValueHint::Url),
		)
		.subcommand(method(GET, GET_ALIAS, "Execute GET request"))
		.subcommand(method(POST, POST_ALIAS, "Execute POST request"))
		.subcommand(method(PUT, PUT_ALIAS, "Execute PUT request"))
		.subcommand(method(DELETE, DELETE_ALIAS, "Execute DELETE request"))
		.subcommand(method(OPTIONS, OPTIONS_ALIAS, "Execute OPTIONS request"))
		.subcommand(method(HEAD, HEAD_ALIAS, "Execute HEAD request"))
		.subcommand(method(PATCH, PATCH_ALIAS, "Execute PATCH request"))
		.subcommand(method(TRACE, TRACE_ALIAS, "Execute TRACE request"))
}

fn method(name: &'static str, alias: &'static str, about: &'static str) -> Command {
	Command::new(name)
		.alias(alias)
		.about(about)
		.arg(
			Arg::new(ENDPOINT)
				.help("Endpoint to call")
				.index(1)
				.required(true)
				.value_parser(endpoint_parser)
				.add(ArgValueCompleter::new(endpoint_completer)),
		)
		.arg(
			Arg::new(PARAMS)
				.short(PARAMS_SHORT)
				.long(PARAMS_LONG)
				.help("Add request parameter (multiple)")
				.value_name("name=value")
				.action(ArgAction::Append)
				.add(ArgValueCompleter::new(param_completer)),
		)
}

fn endpoint_parser(endpoint: &str) -> Result<String, &'static str> {
	let args: Vec<_> = args().skip(1).collect();
	let Some(spec_url) = args
		.windows(2)
		.find(|p| p[0] == format!("-{SPEC_URL_SHORT}") || p[0] == format!("--{SPEC_URL_LONG}"))
		.map(|p| &p[1])
	else {
		return Err("Specification URL is required");
	};

	let method = args
		.chunks(2)
		.find(|p| !p[0].starts_with('-'))
		.map(|p| p[0].as_ref())
		.expect("Method should be valid");

	let Ok(spec) = fetch_spec_blocking(spec_url) else {
		return Err("Failed to fetch specification");
	};

	let valid =
		spec.endpoints.into_iter().filter(|(e, _)| e == endpoint).any(|(_, p)| match method {
			GET | GET_ALIAS => p.get.is_some(),
			POST | POST_ALIAS => p.post.is_some(),
			PUT | PUT_ALIAS => p.put.is_some(),
			DELETE | DELETE_ALIAS => p.delete.is_some(),
			OPTIONS | OPTIONS_ALIAS => p.options.is_some(),
			HEAD | HEAD_ALIAS => p.head.is_some(),
			PATCH | PATCH_ALIAS => p.patch.is_some(),
			TRACE | TRACE_ALIAS => p.trace.is_some(),
			_ => unreachable!(),
		});

	if valid { Ok(endpoint.to_owned()) } else { Err("Endpoint does not exist") }
}

fn endpoint_completer(endpoint: &OsStr) -> Vec<CompletionCandidate> {
	let Some(endpoint) = &endpoint.to_str().map(|e| e.replace(['\\', '\'', '"'], "")) else {
		return vec!["Endpoint should be valid UTF-8".into()];
	};

	let args: Vec<_> = args().skip(1).collect();
	let Some(spec_url) = args
		.windows(2)
		.find(|p| p[0] == format!("-{SPEC_URL_SHORT}") || p[0] == format!("--{SPEC_URL_LONG}"))
		.map(|p| &p[1])
	else {
		return vec!["Specification URL is required for completion".into()];
	};

	let method = args
		.chunks(2)
		.find(|p| !p[0].starts_with('-'))
		.map(|p| p[0].as_ref())
		.expect("Method should be valid");

	let Ok(spec) = fetch_spec_blocking(spec_url) else {
		return vec!["Failed to fetch specification".into()];
	};

	spec.endpoints
		.iter()
		.filter(|(p, _)| p.starts_with(endpoint))
		.filter_map(|(p, e)| match method {
			GET | GET_ALIAS => e.get.as_ref().map(|e| (p, e)),
			POST | POST_ALIAS => e.post.as_ref().map(|e| (p, e)),
			PUT | PUT_ALIAS => e.put.as_ref().map(|e| (p, e)),
			DELETE | DELETE_ALIAS => e.delete.as_ref().map(|e| (p, e)),
			OPTIONS | OPTIONS_ALIAS => e.options.as_ref().map(|e| (p, e)),
			HEAD | HEAD_ALIAS => e.head.as_ref().map(|e| (p, e)),
			PATCH | PATCH_ALIAS => e.patch.as_ref().map(|e| (p, e)),
			TRACE | TRACE_ALIAS => e.trace.as_ref().map(|e| (p, e)),
			_ => None,
		})
		.map(|(e, m)| {
			let mut help = m.summ.clone().unwrap_or_default();
			if m.deprecated {
				help.push_str(" (deprecated)");
			}
			CompletionCandidate::new(e).help(Some(help.into()))
		})
		.collect()
}

fn param_completer(param: &OsStr) -> Vec<CompletionCandidate> {
	let args: Vec<_> = args().skip(1).collect();
	let Some(spec_url) = args
		.windows(2)
		.find(|p| p[0] == format!("-{SPEC_URL_SHORT}") || p[0] == format!("--{SPEC_URL_LONG}"))
		.map(|p| &p[1])
	else {
		return vec!["Specification URL is required for completion".into()];
	};

	let (method, endpoint) = args
		.chunks(2)
		.find(|p| !p[0].starts_with('-'))
		.map(|p| (p[0].as_ref(), p[1].replace(['\\', '\'', '"'], "")))
		.expect("Method and endpoint should be valid");

	let param = param.to_str().unwrap();
	let passed: HashSet<_> = args
		.windows(2)
		.filter(|p| p[0] == format!("-{PARAMS_SHORT}") || p[0] == format!("--{PARAMS_LONG}"))
		.filter_map(|p| p[1].split('=').next().map(str::to_owned))
		.collect();

	let Ok(spec) = fetch_spec_blocking(spec_url) else {
		return vec!["Failed to fetch specification".into()];
	};

	spec.endpoints
		.into_iter()
		.find(|(p, _)| p == &endpoint)
		.and_then(|(_, p)| match method {
			GET | GET_ALIAS => p.get,
			POST | POST_ALIAS => p.post,
			PUT | PUT_ALIAS => p.put,
			DELETE | DELETE_ALIAS => p.delete,
			OPTIONS | OPTIONS_ALIAS => p.options,
			HEAD | HEAD_ALIAS => p.head,
			PATCH | PATCH_ALIAS => p.patch,
			TRACE | TRACE_ALIAS => p.trace,
			_ => None,
		})
		.map(|o| {
			o.params
				.iter()
				.filter(|p| !passed.contains(&p.name))
				.filter(|p| p.name.starts_with(param))
				.map(|p| {
					let kind = p.schema.kind.clone().or_else(|| p.schema.ref_.clone());
					let mut help = kind.unwrap_or("unknown".to_owned());
					if p.required {
						help.push_str(" (required)");
					}
					CompletionCandidate::new(format!("{}=", p.name)).help(Some(help.into()))
				})
				.collect()
		})
		.unwrap_or_default()
}
