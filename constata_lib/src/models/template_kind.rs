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

macro_rules! translation_tuples {
  ( $($field:ident),* ) => {
    (
      $( i18n::t!(i18n::Lang::En, $field), i18n::t!(i18n::Lang::Es, $field), )* 
    )
  }
}

impl TemplateKind {
  pub fn default_schema(&self) -> TemplateSchema {
    let ts = match self {
      TemplateKind::Diploma => translation_tuples!(
        diploma_schema_name, diploma_schema_email, diploma_schema_recipient_identification,
        diploma_schema_custom_text, diploma_schema_motive, diploma_schema_date,
        diploma_schema_place, diploma_schema_shared_text
      ),
      TemplateKind::Attendance => translation_tuples!(
        attendance_schema_name, attendance_schema_email, attendance_schema_recipient_identification,
        attendance_schema_custom_text, attendance_schema_motive, attendance_schema_date,
        attendance_schema_place, attendance_schema_shared_text
      ),
      TemplateKind::Badge => translation_tuples!(
        badge_schema_name, badge_schema_email, badge_schema_recipient_identification,
        badge_schema_custom_text, badge_schema_motive, badge_schema_date,
        badge_schema_place, badge_schema_shared_text
      ),
    };

    vec![
      TemplateSchemaField::new("name", false, false, ts.0, ts.1),
      TemplateSchemaField::new("email", true, false, ts.2, ts.3),
      TemplateSchemaField::new("recipient_identification", true, false, ts.4, ts.5),
      TemplateSchemaField::new("custom_text", true, false, ts.6, ts.7),
      TemplateSchemaField::new("motive", false, true, ts.8, ts.9),
      TemplateSchemaField::new("date", true, true, ts.10, ts.11),
      TemplateSchemaField::new("place", true, true, ts.12, ts.13),
      TemplateSchemaField::new("shared_text", true, true, ts.14, ts.15),
    ]
  }
}

impl sqlx::postgres::PgHasArrayType for TemplateKind {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_template_kind")
  }
}
