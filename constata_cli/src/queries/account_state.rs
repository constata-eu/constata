use super::*;

#[derive(serde::Serialize, clap::Args)]
pub struct AccountState { }

impl AccountState {
  pub fn run(&self, client: &Client) -> ClientResult<gql_types::AccountState> {
    client.simple(self, "AccountState", &format!("query{{
      AccountState(id: 1) {{
        {}
      }}
    }}", gql_fields::ACCOUNT_STATE))
  }
}
