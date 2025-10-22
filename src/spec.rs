use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Spec {
	#[serde(rename = "paths")]
	pub rpcs: HashMap<String, Rpcs>,
	#[serde(rename = "components")]
	pub comps: Components,
}

#[derive(Debug, Deserialize)]
pub struct Rpcs {
	pub get: Option<Rpc>,
	pub post: Option<Rpc>,
	pub put: Option<Rpc>,
	pub delete: Option<Rpc>,
	pub options: Option<Rpc>,
	pub head: Option<Rpc>,
	pub patch: Option<Rpc>,
	pub trace: Option<Rpc>,
}

#[derive(Debug, Deserialize)]
pub struct Rpc {
	#[serde(default, rename = "parameters")]
	pub params: Vec<Param>,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
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

#[derive(PartialEq, Eq, Debug, Deserialize)]
pub struct Schema {
	#[serde(rename = "$ref")]
	pub ptr: Option<String>,
	#[serde(rename = "type")]
	pub kind: Option<SchemaKind>,
	pub items: Option<Box<Schema>>,
	#[serde(rename = "enum")]
	pub variants: Option<Vec<String>>,
	pub pattern: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(untagged)]
pub enum SchemaKind {
	Single(String),
	Multiple(Vec<String>),
}

impl Spec {
	pub fn traverse_schema<'a>(&'a self, schema: &'a Schema) -> Option<&'a Schema> {
		let Some(p) = &schema.ptr else { return Some(schema) };
		let s = p.strip_prefix("#/components/schemas/")?;
		self.traverse_schema(self.comps.schemas.get(s)?)
	}

	pub fn describe_schema(&self, schema: &Schema) -> String {
		let Some(schema) = self.traverse_schema(schema) else { return "unknown".to_owned() };
		if let Some(e) = &schema.variants {
			return e.join("|");
		}

		if let Some(r) = &schema.pattern {
			return format!("regex<{r}>");
		}

		if let Some(k) = &schema.kind {
			return match k {
				SchemaKind::Single(s) => match s.as_ref() {
					"array" => schema.items.as_ref().map_or_else(
						|| "array<unknown>".to_owned(),
						|a| format!("array<{}>", self.describe_schema(a)),
					),
					"string" => "str".to_owned(),
					"integer" => "int".to_owned(),
					"boolean" => "bool".to_owned(),
					_ => s.to_owned(),
				},
				SchemaKind::Multiple(v) => v.join("|"),
			};
		}

		"unknown".to_owned()
	}
}

#[derive(Debug, Deserialize)]
pub struct Components {
	pub schemas: HashMap<String, Schema>,
}
