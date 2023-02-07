use super::*;
use juniper::GraphQLEnum;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum)]
#[sqlx(type_name = "template_kind", rename_all = "lowercase")]
pub enum TemplateKind {
  Diploma,
  Attendance,
  Invitation,
}

impl sqlx::postgres::PgHasArrayType for CertificateKind {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_template_kind")
  }
}
