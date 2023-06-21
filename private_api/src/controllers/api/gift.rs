use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "A gift")]
pub struct Gift {
  id: i32,
  org_id: i32,
  created_at: UtcDateTime,
  tokens: i32,
  reason: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct GiftFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  tokens_eq: Option<i32>,
  tokens_gt: Option<i32>,
  tokens_lt: Option<i32>,
}

#[rocket::async_trait]
impl Showable<db::Gift, GiftFilter> for Gift {
  fn sort_field_to_order_by(field: &str) -> Option<GiftOrderBy> {
    match field {
      "id" => Some(GiftOrderBy::Id),
      "orgId" => Some(GiftOrderBy::OrgId),
      "tokens" => Some(GiftOrderBy::Tokens),
      "createdAt" => Some(GiftOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(f: GiftFilter) -> SelectGift {
    SelectGift{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: f.org_id_eq,
      tokens_eq: into_decimal(f.tokens_eq),
      tokens_gt: into_decimal(f.tokens_gt),
      tokens_lt: into_decimal(f.tokens_lt),
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: db::Gift ) -> ConstataResult<Self> {
    Ok(Gift {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      created_at: d.attrs.created_at,
      tokens: d.attrs.tokens.to_i32().unwrap_or(0),
      reason: Some(d.attrs.reason),
    })
  }
}

impl Gift {
  pub async fn create_gift(
    context: &Context, org_id: i32, tokens: i32, reason: String
  ) -> FieldResult<Gift> {
    let db_gift = context.site.gift()
      .give_and_trigger_updates(org_id, Decimal::new(tokens.into(), 0), &reason)
      .await?;

    Ok(Gift::db_to_graphql(db_gift).await?)
  }

}
