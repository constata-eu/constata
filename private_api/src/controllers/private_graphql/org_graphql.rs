use super::*;


#[derive(GraphQLObject, Clone, Debug)]
#[graphql(description = "A org using the system")]
pub struct Org {
  id: i32,
  public_name: Option<String>,
  logo_url: Option<String>,
  subscription_id: Option<i32>,
  stripe_customer_id: Option<String>,
  token_balance: i32,
  funded_count: i32,
  funded_tokens: i32,
  parked_count: i32,
  parked_tokens: i32,
  total_document_count: i32,
  total_document_tokens: i32,
  missing_tokens: i32,
  first_parked_date: Option<UtcDateTime>,
  last_parked_date: Option<UtcDateTime>,
  pending_invoice_link_url: Option<String>,
  name_for_on_behalf_of: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct OrgFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  public_name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<org::Org, OrgFilter> for Org {
  fn sort_field_to_order_by(field: &str) -> Option<OrgOrderBy> {
    match field {
      "id" => Some(OrgOrderBy::Id),
      "publicName" => Some(OrgOrderBy::PublicName),
      _ => None,
    }
  }

  fn filter_to_select(f: OrgFilter) -> SelectOrg {
    SelectOrg{
      id_in: f.ids,
      id_eq: f.id_eq,
      public_name_ilike: into_like_search(f.public_name_like),
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: org::Org ) -> MyResult<Self> {
    let account_state = d.account_state().await?;
    let (first_parked_date, last_parked_date) = d.date_first_and_last_document_parked().await;
    let name_for_on_behalf_of = d.name_for_on_behalf_of().await?;

    Ok(Org {
      id: d.attrs.id,
      public_name: d.attrs.public_name,
      logo_url: d.attrs.logo_url,
      subscription_id: d.attrs.subscription_id,
      stripe_customer_id: d.attrs.stripe_customer_id,
      token_balance: account_state.token_balance.to_i32().unwrap_or(0),
      total_document_count: account_state.total_document_count.to_i32().unwrap_or(0),
      total_document_tokens: account_state.total_document_tokens.to_i32().unwrap_or(0),
      parked_count: account_state.parked_count.to_i32().unwrap_or(0),
      parked_tokens: account_state.parked_tokens.to_i32().unwrap_or(0),
      funded_count: account_state.funded_count.to_i32().unwrap_or(0),
      funded_tokens: account_state.funded_tokens.to_i32().unwrap_or(0),
      missing_tokens: account_state.missing.to_i32().unwrap_or(0),
      first_parked_date,
      last_parked_date,
      pending_invoice_link_url: account_state.pending_invoice_link_url,
      name_for_on_behalf_of,
    })
  }
}

impl Org {
  pub async fn update_org(
    context: &Context, id: i32, logo_url: Option<String>, public_name: Option<String>
  ) -> FieldResult<Org> {
    let org = context.site.org().find(&id).await?
      .update()
      .logo_url(logo_url)
      .public_name(public_name)
      .save().await?;
    
    Ok(Org::db_to_graphql(org).await?)
  }
}