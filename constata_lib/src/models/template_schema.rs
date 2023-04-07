#[derive(Debug, juniper::GraphQLObject, serde::Deserialize, serde::Serialize)]
pub struct TemplateSchemaField {
  pub name: String,
  pub optional: bool,
  pub common: bool,
  pub label: Option<String>,
  pub help: Option<String>,
  pub sample: Option<String>,
}

impl TemplateSchemaField {
  pub fn new(name: &str, optional: bool, common: bool) -> Self{
    Self {
      name:name.to_string(),
      optional,
      common,
      label: None,
      help: None,
      sample: None,
    }
  }
}

pub type TemplateSchema = Vec<TemplateSchemaField>;
