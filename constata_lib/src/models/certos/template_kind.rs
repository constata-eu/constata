use crate::models::*;
use juniper::GraphQLEnum;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum, clap::ValueEnum)]
#[sqlx(type_name = "template_kind", rename_all = "lowercase")]
#[serde(rename_all = "UPPERCASE")]
pub enum TemplateKind {
  Diploma,
  Attendance,
  Badge,
}

impl TemplateKind {
  pub fn default_schema(&self) -> TemplateSchema {
    vec![
      TemplateSchemaField::new("name", false, false),
      TemplateSchemaField::new("email", true, false),
      TemplateSchemaField::new("recipient_identification", true, false),
      TemplateSchemaField::new("custom_text", true, false),
      TemplateSchemaField::new("motive", false, true),
      TemplateSchemaField::new("date", true, true),
      TemplateSchemaField::new("place", true, true),
      TemplateSchemaField::new("shared_text", true, true),
    ]
  }
}

impl sqlx::postgres::PgHasArrayType for TemplateKind {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_template_kind")
  }
}
