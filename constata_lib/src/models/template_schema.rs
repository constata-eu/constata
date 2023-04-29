use i18n::Lang; 

#[derive(Debug, juniper::GraphQLObject, serde::Deserialize, serde::Serialize)]
pub struct TemplateSchemaField {
  pub name: String,
  pub optional: bool,
  pub common: bool,
  pub label: Option<String>,
  pub label_es: Option<String>,
  pub help: Option<String>,
  pub sample: Option<String>,
}

impl TemplateSchemaField {
  pub fn new(name: &str, optional: bool, common: bool, label: String, label_es: String) -> Self{
    Self {
      name:name.to_string(),
      optional,
      common,
      label: Some(label),
      label_es: Some(label_es),
      help: None,
      sample: None,
    }
  }

  pub fn i18n_label(&self, l: Lang) -> Option<&str> {
    let local = match l {
      Lang::En => self.label.as_deref(),
      Lang::Es => self.label_es.as_deref(),
    };
    local.or(self.label.as_deref())
  }
}

pub type TemplateSchema = Vec<TemplateSchemaField>;
