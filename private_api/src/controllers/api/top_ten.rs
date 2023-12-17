use super::*;

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(description = "A top10")]
pub struct TopTen {
  id: i32,
  token_balance: i32,
  funded_count: i32,
  funded_tokens: i32,
  parked_count: i32,
  parked_tokens: i32,
  missing_tokens: i32,
}


impl TopTen {
    pub async fn get_top(context: &Context) -> ConstataResult<Vec<Self>> {
      let mut list = vec![];
      for o in context.site.org().top_ten().all().await? {
        list.push(Self::db_to_graphql(o).await?);
      }
  
      Ok(list)
    }
  
    pub async fn db_to_graphql(d: db::Org ) -> ConstataResult<TopTen> {
      let account_state = d.account_state().await?;
      Ok(TopTen {
        id: d.attrs.id,
        token_balance: account_state.token_balance.to_i32().unwrap_or(0),
        parked_count: account_state.parked_count.to_i32().unwrap_or(0),
        parked_tokens: account_state.parked_tokens.to_i32().unwrap_or(0),
        funded_count: account_state.funded_count.to_i32().unwrap_or(0),
        funded_tokens: account_state.funded_tokens.to_i32().unwrap_or(0),
        missing_tokens: account_state.missing.to_i32().unwrap_or(0),
      })
    }
  }
  
