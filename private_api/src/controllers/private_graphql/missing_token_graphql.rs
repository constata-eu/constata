use super::*;


#[derive(GraphQLObject, Clone, Debug)]
#[graphql(description = "A top of parked documents")]
pub struct MissingToken {
  id: i32,
  token_balance: i32,
  funded_count: i32,
  funded_tokens: i32,
  parked_count: i32,
  parked_tokens: i32,
  missing_tokens: i32,
  invoice_url: Option<String>,
  invoice_link_url: Option<String>,
  last_parked_date: Option<UtcDateTime>,
}


impl MissingToken {
    pub async fn db_to_graphql(d: org::Org ) -> MyResult<MissingToken> {
      let account_state = d.account_state().await?;
      let (_, last_parked_date) = d.date_first_and_last_document_parked().await;
      let invoice_url = if account_state.invoices.len() > 0 {
        Some(account_state.invoices[0].attrs.url.clone())
      } else {
        None
      };
  
      Ok(MissingToken {
        id: d.attrs.id,
        token_balance: account_state.token_balance.to_i32().unwrap_or(0),
        parked_count: account_state.parked_count.to_i32().unwrap_or(0),
        parked_tokens: account_state.parked_tokens.to_i32().unwrap_or(0),
        funded_count: account_state.funded_count.to_i32().unwrap_or(0),
        funded_tokens: account_state.funded_tokens.to_i32().unwrap_or(0),
        missing_tokens: account_state.missing.to_i32().unwrap_or(0),
        invoice_url,
        invoice_link_url: account_state.pending_invoice_link_url,
        last_parked_date,
      })
    }
  
    pub async fn count(context: &Context) -> FieldResult<ListMetadata> {
      let count = context.site.org().orgs_that_are_missing_tokens_count().await?.to_i32().unwrap_or(0);
      Ok(ListMetadata{count})
    }
  
    pub async fn collection(context: &Context, page: Option<i32>, per_page: Option<i32>,) -> FieldResult<Vec<Self>> {
      let limit: i32 = per_page.unwrap_or(DEFAULT_PER_PAGE);
      if limit >= 500 {
        return Err(FieldError::new(
          "Invalid pagination",
          graphql_value!({ "internal_error": "Invalid pagination" })
        ));
      }
      let offset: i32 = page.unwrap_or(DEFAULT_PAGE) * limit;
  
      let db_orgs = context.site.org()
        .missing_tokens(limit.into(), offset.into())
        .all().await?;
  
      let mut graphql_orgs = vec![];
      for o in db_orgs.into_iter() {
        graphql_orgs.push(Self::db_to_graphql(o).await?);
      }
  
      Ok(graphql_orgs)
    }
  }
  
  