use super::*;
use juniper::GraphQLEnum;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum)]
#[sqlx(type_name = "deletion_reason", rename_all = "lowercase")]
pub enum DeletionReason {
  UserRequest,
  ConstataDecision,
  Inactivity,
}

impl sqlx::postgres::PgHasArrayType for DeletionReason {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_deletion_reason")
  }
}
