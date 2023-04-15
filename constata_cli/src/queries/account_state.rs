use super::*;
use gql_types::account_state_graphql::AccountState as Model;

#[derive(serde::Serialize, clap::Args)]
pub struct AccountState { }

impl AccountState {
  pub fn run(&self, client: &Client) -> ClientResult<Model> {
    client.simple(self, "AccountState", &format!("query{{
      AccountState(id: 1) {{
        {}
      }}
    }}", gql_fields::ACCOUNT_STATE))
  }
}
