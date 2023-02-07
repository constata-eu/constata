use super::*;


#[derive(GraphQLObject)]
#[graphql(description = "Subscriptions")]
pub struct Subscription {
  id: i32,
  org_id: i32,
  created_at: UtcDateTime,
  invoicing_day: i32,
  is_active: bool,
  plan_name: Option<String>,
  max_monthly_gift: Option<i32>,
  monthly_gift_remainder: Option<i32>,
  required_token_purchase: Option<i32>,
  price_per_token: Option<i32>,
  default_payment_source: Option<PaymentSource>,
  stripe_subscription_id: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct SubscriptionFilter {
  ids: Option<Vec<i32>>,
  org_id_eq: Option<i32>,
}

#[rocket::async_trait]
impl Showable<subscription::Subscription, SubscriptionFilter> for Subscription {
  fn sort_field_to_order_by(field: &str) -> Option<SubscriptionOrderBy> {
    match field {
      "id" => Some(SubscriptionOrderBy::Id),
      "orgId" => Some(SubscriptionOrderBy::OrgId),
      "planName" => Some(SubscriptionOrderBy::PlanName),
      "defaultPaymentSource" => Some(SubscriptionOrderBy::DefaultPaymentSource),
      "maxMonthlyGift" => Some(SubscriptionOrderBy::MaxMonthlyGift),
      "pricePerToken" => Some(SubscriptionOrderBy::PricePerToken),
      "isActive" => Some(SubscriptionOrderBy::IsActive),
      _ => None,
    }
  }

  fn filter_to_select(f: SubscriptionFilter) -> SelectSubscription {
    SelectSubscription{
      id_in: f.ids,
      org_id_eq: f.org_id_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: subscription::Subscription ) -> MyResult<Self> {
    let price_per_token = d.attrs.price_per_token * Decimal::new(100, 0);
    let monthly_gift_remainder = Some(d.monthly_gift_remainder().await?.to_i32().unwrap_or(0));
    Ok(Subscription {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      created_at: d.attrs.created_at,
      invoicing_day: d.attrs.invoicing_day,
      is_active: d.attrs.is_active,
      plan_name: Some(d.attrs.plan_name),
      max_monthly_gift: Some(d.attrs.max_monthly_gift.to_i32().unwrap_or(0)),
      required_token_purchase: Some(d.attrs.required_token_purchase.to_i32().unwrap_or(0)),
      price_per_token: Some(price_per_token.to_i32().unwrap_or(0)),
      default_payment_source: d.attrs.default_payment_source,
      stripe_subscription_id: d.attrs.stripe_subscription_id,
      monthly_gift_remainder,
    })
  }
}

impl Subscription {
  pub async fn update_subscription(
    context: &Context, id: i32, max_monthly_gift: i32, price_per_token: i32, is_active: bool
  ) -> FieldResult<Subscription> {
    let db_subscription = context.site.subscription().find(&id).await?;
    let updated_subscription = db_subscription.update()
      .max_monthly_gift(Decimal::new(max_monthly_gift.into(), 0))
      .price_per_token(Decimal::new(price_per_token.into(), 2))
      .is_active(is_active)
      .save().await?;

    Ok(Subscription::db_to_graphql(updated_subscription).await?)
  }
}