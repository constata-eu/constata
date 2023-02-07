use crate::models::*;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[sqlx(type_name = "outgoing_email_message_kind", rename_all = "lowercase")]
pub enum OutgoingEmailMessageKind {
  Welcome,
  KycRequestReceived,
  EmailVerification,
}

impl sqlx::postgres::PgHasArrayType for OutgoingEmailMessageKind {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_outgoing_email_message_kind")
  }
}
