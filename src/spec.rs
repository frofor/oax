use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Spec {
	#[serde(rename = "paths")]
	pub endpoints: HashMap<String, Endpoint>,
	#[serde(rename = "components")]
	pub comps: Components,
}

#[derive(Debug, Deserialize)]
pub struct Endpoint {
	pub get: Option<Method>,
	pub post: Option<Method>,
	pub put: Option<Method>,
	pub delete: Option<Method>,
	pub options: Option<Method>,
	pub head: Option<Method>,
	pub patch: Option<Method>,
	pub trace: Option<Method>,
}

#[derive(Debug, Deserialize)]
pub struct Method {
	#[serde(rename = "summary")]
	pub summ: Option<String>,
	#[serde(default, rename = "parameters")]
	pub params: Vec<Param>,
	#[serde(default)]
	pub deprecated: bool,
}

#[derive(Debug, Deserialize)]
pub struct Param {
	pub name: String,
	#[serde(rename = "in")]
	pub kind: ParamKind,
	#[serde(default)]
	pub required: bool,
	pub schema: Schema,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamKind {
	Query,
	Path,
	Header,
	Cookie,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
	#[serde(rename = "$ref")]
	pub ref_: Option<String>,
	#[serde(rename = "type")]
	pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Components {
	pub schemas: HashMap<String, Schema>,
}
