use super::*;
use juniper::GraphQLEnum;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum)]
#[sqlx(type_name = "payment_source", rename_all = "lowercase")]
pub enum PaymentSource {
  BankBbva,
  Stripe,
  BtcPay,
}

impl sqlx::postgres::PgHasArrayType for PaymentSource {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_payment_source")
  }
}
