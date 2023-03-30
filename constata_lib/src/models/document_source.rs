use super::*;
use juniper::GraphQLEnum;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum)]
#[sqlx(type_name = "document_source", rename_all = "lowercase")]
pub enum DocumentSource {
  Email,
  Api,
  Telegram, // Legacy document source from when we had a telegram robot.
  Internal,
}

impl sqlx::postgres::PgHasArrayType for DocumentSource {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_document_source")
  }
}
