#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

pub(crate) mod api;
pub(crate) mod spec;

use api::get;
use api::spec;
use inquire::Select;
use inquire::Text;
use inquire::error::InquireResult;
use inquire::ui::Attributes;
use inquire::ui::Color;
use inquire::ui::RenderConfig;
use inquire::ui::StyleSheet;
use inquire::ui::Styled;
use serde_json::Value;
use spec::Param;
use spec::ParamKind;
use spec::Schema;
use spec::SchemaKind;
use spec::Spec;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Stdout;
use std::io::Write as _;
use std::io::stdout;
use std::iter::once;
use std::process::exit;

#[tokio::main]
async fn main() {
	let stdout = &mut stdout();
	clear(stdout);

	let cfg = RenderConfig::default()
		.with_text_input(StyleSheet::new().with_fg(Color::DarkGreen))
		.with_prompt_prefix(Styled::new(">").with_fg(Color::LightGreen))
		.with_option(StyleSheet::new().with_fg(Color::DarkGreen))
		.with_selected_option(Some(
			StyleSheet::new()
				.with_fg(Color::Black)
				.with_bg(Color::DarkGreen)
				.with_attr(Attributes::BOLD),
		))
		.with_highlighted_option_prefix(Styled::new("▶").with_fg(Color::DarkGreen))
		.with_scroll_up_prefix(Styled::new("▲").with_fg(Color::DarkGreen))
		.with_scroll_down_prefix(Styled::new("▼").with_fg(Color::DarkGreen));

	let spec_url = &Text::new("Url:").with_render_config(cfg).prompt().unwrap_or_else(|_| exit(1));

	clear(stdout);

	let spec = &spec(spec_url).await.unwrap_or_else(|e| {
		eprintln!("Failed to read specification: {e}");
		exit(1);
	});

	let ctx = &mut PromptCtx::new(spec, &cfg, stdout);

	let Ok((method, endpoint)) = prompt_rpc(spec, ctx) else { exit(1) };

	let rpc = match method {
		Method::Get => spec.rpcs.get(endpoint).unwrap().get.as_ref().unwrap(),
		Method::Post => spec.rpcs.get(endpoint).unwrap().post.as_ref().unwrap(),
		Method::Put => spec.rpcs.get(endpoint).unwrap().put.as_ref().unwrap(),
		Method::Delete => spec.rpcs.get(endpoint).unwrap().delete.as_ref().unwrap(),
		Method::Patch => spec.rpcs.get(endpoint).unwrap().patch.as_ref().unwrap(),
		Method::Head => spec.rpcs.get(endpoint).unwrap().head.as_ref().unwrap(),
		Method::Options => spec.rpcs.get(endpoint).unwrap().options.as_ref().unwrap(),
		Method::Trace => spec.rpcs.get(endpoint).unwrap().trace.as_ref().unwrap(),
	};

	let Ok(param_values) = &prompt_params(&rpc.params, ctx) else { exit(1) };

	let base_url = extract_base_url(spec_url).unwrap();
	let url = &format!("{base_url}{}", build_url(endpoint, param_values, &rpc.params));

	match method {
		Method::Get => {
			let res = get(url).await.unwrap_or_else(|e| {
				eprintln!("Request failed: {e}");
				exit(1);
			});
			let body = res.text().await.unwrap_or_else(|e| {
				eprintln!("Decode failed: {e}");
				exit(1);
			});
			println!("{body}");
		}
		_ => unimplemented!(),
	}
}

fn prompt_rpc<'a>(spec: &'a Spec, ctx: &mut PromptCtx) -> InquireResult<(Method, &'a str)> {
	let actions = spec
		.rpcs
		.iter()
		.flat_map(|(e, r)| {
			[
				r.get.as_ref().map(|_| RpcAction::new(Method::Get, e)),
				r.post.as_ref().map(|_| RpcAction::new(Method::Post, e)),
				r.put.as_ref().map(|_| RpcAction::new(Method::Put, e)),
				r.delete.as_ref().map(|_| RpcAction::new(Method::Delete, e)),
				r.patch.as_ref().map(|_| RpcAction::new(Method::Patch, e)),
				r.head.as_ref().map(|_| RpcAction::new(Method::Head, e)),
				r.options.as_ref().map(|_| RpcAction::new(Method::Options, e)),
				r.trace.as_ref().map(|_| RpcAction::new(Method::Trace, e)),
			]
		})
		.flatten()
		.collect();

	let action = Select::new("Rpc:", actions)
		.with_render_config(*ctx.cfg)
		.without_help_message()
		.prompt()?;

	clear(ctx.stdout);

	Ok((action.method, action.endpoint))
}

fn prompt_params(params: &[Param], ctx: &mut PromptCtx) -> InquireResult<HashMap<String, Value>> {
	let mut values = params
		.iter()
		.filter(|p| p.required)
		.map(|p| {
			let value_from = |k| match k {
				"boolean" => Value::Bool(false),
				"integer" => Value::Number(0.into()),
				"string" => Value::String(String::new()),
				"null" => Value::Null,
				_ => unreachable!(),
			};

			let value = match ctx.spec.traverse_schema(&p.schema).unwrap().kind.as_ref().unwrap() {
				SchemaKind::Single(k) => value_from(k),
				SchemaKind::Multiple(k) => value_from(k.first().unwrap()),
			};

			(p.name.clone(), value)
		})
		.collect();

	if params.is_empty() {
		return Ok(values);
	}

	let mut param = &params[0];

	loop {
		let longest = params.iter().max_by_key(|p| p.name.len()).map(|p| p.name.len()).unwrap();

		let actions: Vec<_> = params
			.iter()
			.map(|p| ParamsAction::Set(p, values.get(&p.name).cloned(), longest))
			.chain(once(ParamsAction::Done))
			.collect();

		let cursor = actions
			.iter()
			.position(|a| matches!(a, ParamsAction::Set(p, ..) if p == &param))
			.unwrap();

		let action = Select::new("Params:", actions)
			.with_render_config(*ctx.cfg)
			.with_starting_cursor(cursor)
			.without_help_message()
			.prompt()?;

		clear(ctx.stdout);

		param = if let ParamsAction::Set(p, ..) = &action { p } else { break };

		match prompt_param(param, values.get(&param.name), ctx)? {
			Some(v) => values.insert(param.name.clone(), v),
			None => values.remove(&param.name),
		};
	}

	Ok(values)
}

fn prompt_param(
	param: &Param,
	value: Option<&Value>,
	ctx: &mut PromptCtx,
) -> InquireResult<Option<Value>> {
	let schema = ctx.spec.traverse_schema(&param.schema).unwrap();

	let Some(items) = &schema.items else { return prompt_prim(param, schema, value, ctx) };

	let mut values = value.map_or_else(Vec::new, |v| v.as_array().unwrap().clone());
	let mut cursor = 0;

	loop {
		let actions: Vec<_> = once(ParamAction::Add)
			.chain(values.iter().map(|v| ParamAction::Remove(v.clone())))
			.chain(once(ParamAction::Done))
			.collect();

		let action = Select::new(&format!("{}:", param.name), actions.clone())
			.with_render_config(*ctx.cfg)
			.with_starting_cursor(cursor)
			.without_help_message()
			.prompt()?;

		clear(ctx.stdout);

		cursor = actions.iter().position(|a| a == &action).unwrap();

		match action {
			ParamAction::Add => {
				if let Some(v) = prompt_prim(param, items, None, ctx)? {
					values.push(v);
				}
			}
			ParamAction::Remove(o) => values.retain(|v| v != &o),
			ParamAction::Done => break,
		}
	}

	Ok(Some(Value::Array(values)))
}

fn prompt_prim(
	param: &Param,
	schema: &Schema,
	initial: Option<&Value>,
	ctx: &mut PromptCtx,
) -> InquireResult<Option<Value>> {
	let schema = ctx.spec.traverse_schema(schema).unwrap();

	let mut actions = Vec::new();

	if let Some(v) = &schema.variants {
		actions.extend(v.iter().map(|v| PrimAction::Str(v)));
	}

	let push_if_missing = |x, xs: &mut Vec<_>| {
		if !xs.contains(&x) {
			xs.push(x);
		}
	};

	let mut push_actions_from = |k| match k {
		"boolean" => {
			push_if_missing(PrimAction::Bool(true), &mut actions);
			push_if_missing(PrimAction::Bool(false), &mut actions);
		}
		"integer" | "string" => push_if_missing(PrimAction::Custom, &mut actions),
		"null" => push_if_missing(PrimAction::Null, &mut actions),
		_ => {}
	};

	match &schema.kind {
		Some(SchemaKind::Single(k)) => push_actions_from(k),
		Some(SchemaKind::Multiple(k)) => k.iter().for_each(|k| push_actions_from(k)),
		None => {}
	}

	if !param.required {
		actions.push(PrimAction::None);
	}

	let cursor = actions
		.iter()
		.position(|a| match (a, initial) {
			(PrimAction::Bool(v), Some(Value::Bool(i))) => v == i,
			(PrimAction::Str(v), Some(Value::String(i))) => v == i,
			(PrimAction::Null, Some(Value::Null)) | (PrimAction::None, None) => true,
			_ => false,
		})
		.unwrap_or_default();

	if actions.len() > 1 {
		let action = Select::new(&format!("{}:", param.name), actions)
			.with_render_config(*ctx.cfg)
			.with_starting_cursor(cursor)
			.without_help_message()
			.prompt()?;

		clear(ctx.stdout);

		match action {
			PrimAction::Custom => {}
			PrimAction::Bool(b) => return Ok(Some(Value::Bool(b))),
			PrimAction::Str(s) => return Ok(Some(Value::String(s.to_owned()))),
			PrimAction::Null => return Ok(Some(Value::Null)),
			PrimAction::None => return Ok(None),
		}
	}

	let initial_str = match initial {
		Some(Value::Number(n)) => &n.to_string(),
		Some(Value::String(s)) => s,
		_ => "",
	};

	let value = Text::new(&format!("{}:", param.name))
		.with_render_config(*ctx.cfg)
		.with_initial_value(initial_str)
		.with_help_message(&ctx.spec.describe_schema(schema))
		.prompt()?;

	clear(ctx.stdout);

	let value_from = |k| match k {
		"integer" => value.parse().map(Value::Number).ok(),
		"string" => Some(Value::String(value)),
		_ => unreachable!(),
	};

	Ok(match schema.kind.as_ref().unwrap() {
		SchemaKind::Single(k) => value_from(k).or_else(|| initial.cloned()),
		SchemaKind::Multiple(k) => value_from(k.first().unwrap()).or_else(|| initial.cloned()),
	})
}

fn clear(stdout: &mut Stdout) {
	print!("\x1B[2J\x1B[1;1H");
	let _ = stdout.flush();
}

fn build_url(endpoint: &str, param_values: &HashMap<String, Value>, params: &[Param]) -> String {
	let url = param_values
		.iter()
		.filter(|(n, _)| params.iter().any(|p| &&p.name == n && p.kind == ParamKind::Path))
		.fold(endpoint.to_owned(), |u, (n, v)| match v {
			Value::Bool(v) => u.replace(&format!("{{{n}}}"), &v.to_string()),
			Value::Number(v) => u.replace(&format!("{{{n}}}"), &v.to_string()),
			Value::String(v) => u.replace(&format!("{{{n}}}"), v),
			_ => unreachable!(),
		});

	let query_params: Vec<_> = param_values
		.iter()
		.filter(|(n, _)| params.iter().any(|p| &&p.name == n && p.kind == ParamKind::Query))
		.flat_map(|(n, v)| match v {
			Value::Array(v) => v.iter().map(|v| format!("{n}={v}")).collect(),
			_ => vec![format!("{n}={v}")],
		})
		.collect();

	if query_params.is_empty() { url } else { format!("{url}?{}", query_params.join("&")) }
}

fn extract_base_url(url: &str) -> Option<&str> {
	const SCHEME_DELIM: &str = "://";
	let scheme_end = url.find(SCHEME_DELIM)? + SCHEME_DELIM.len();
	Some(&url[..url[scheme_end..].find('/')? + scheme_end])
}

struct PromptCtx<'a> {
	pub spec: &'a Spec,
	pub cfg: &'a RenderConfig<'a>,
	pub stdout: &'a mut Stdout,
}

impl<'a> PromptCtx<'a> {
	pub fn new(spec: &'a Spec, cfg: &'a RenderConfig<'a>, stdout: &'a mut Stdout) -> Self {
		Self { spec, cfg, stdout }
	}
}

struct RpcAction<'a> {
	pub method: Method,
	pub endpoint: &'a str,
}

impl<'a> RpcAction<'a> {
	fn new(method: Method, endpoint: &'a str) -> Self {
		Self { method, endpoint }
	}
}

impl Display for RpcAction<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		const LONGEST_METHOD: usize = "OPTIONS".len();
		write!(f, "{:<LONGEST_METHOD$} {}", self.method.to_string(), self.endpoint)
	}
}

enum Method {
	Get,
	Post,
	Put,
	Delete,
	Patch,
	Head,
	Options,
	Trace,
}

impl Display for Method {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Get => write!(f, "GET"),
			Self::Post => write!(f, "POST"),
			Self::Put => write!(f, "PUT"),
			Self::Delete => write!(f, "DELETE"),
			Self::Patch => write!(f, "PATCH"),
			Self::Head => write!(f, "HEAD"),
			Self::Options => write!(f, "OPTIONS"),
			Self::Trace => write!(f, "TRACE"),
		}
	}
}

#[derive(PartialEq, Eq)]
enum ParamsAction<'a> {
	Set(&'a Param, Option<Value>, usize),
	Done,
}

impl Display for ParamsAction<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Set(p, v, i) => {
				let value = if let Some(v) = v { &v.to_string() } else { "undefined" };
				write!(f, "{:<i$} = {value}", p.name)
			}
			Self::Done => write!(f, "[Done]"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
enum ParamAction {
	Add,
	Remove(Value),
	Done,
}

impl Display for ParamAction {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Add => write!(f, "[Add]"),
			Self::Remove(v) => write!(f, "[Remove] {v}"),
			Self::Done => write!(f, "[Done]"),
		}
	}
}

#[derive(PartialEq, Eq)]
enum PrimAction<'a> {
	Custom,
	Bool(bool),
	Str(&'a str),
	Null,
	None,
}

impl Display for PrimAction<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Custom => write!(f, "[Custom]"),
			Self::Bool(b) => write!(f, "{b}"),
			Self::Str(s) => write!(f, "\"{s}\""),
			Self::Null => write!(f, "null"),
			Self::None => write!(f, "undefined"),
		}
	}
}
