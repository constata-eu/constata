use crate::error::Result;
use super::*;

model!{
  state: Site,
  table: subscriptions,
  struct Subscription {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    created_at: UtcDateTime,
    invoicing_day: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(boolean)]
    is_active: bool,
    #[sqlx_model_hints(varchar)]
    plan_name: String,
    #[sqlx_model_hints(decimal)]
    max_monthly_gift: Decimal,
    required_token_purchase: Decimal,
    #[sqlx_model_hints(decimal)]
    price_per_token: Decimal,
    #[sqlx_model_hints(varchar)]
    stripe_subscription_id: Option<String>,
    #[sqlx_model_hints(payment_source)]
    default_payment_source: Option<PaymentSource>,
  }
}

impl Subscription {
  pub async fn monthly_gift_remainder(&self) -> sqlx::Result<Decimal> {
    use chrono::Datelike;
    let given = self.state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COALESCE(SUM(tokens), 0)::decimal as "given!" FROM gifts WHERE EXTRACT(month FROM created_at) = $1 AND org_id = $2"#,
      Utc::now().month() as i32,
      self.attrs.org_id,
    )).await?;

    Ok( std::cmp::max(self.attrs.max_monthly_gift - given, Decimal::ZERO) )
  }

  pub async fn claim_monthly_gift(&self, up_to: Decimal) -> Result<Option<Gift>> {
    let remainder = self.monthly_gift_remainder().await?;

    if remainder <= Decimal::ZERO {
      return Ok(None);
    }

    let amount = up_to.min(remainder);

    Ok(Some(
      self.state.gift().build(*self.org_id(), amount, "Digital Inclusion").save().await?
    ))
  }
}
