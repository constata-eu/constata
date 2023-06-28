use super::*;
use gql_types::AccountState;

#[derive(serde::Serialize, clap::Args)]
pub struct UpdateWebCallbacksUrl {
  #[arg(help="Your new web callbacks URL")]
  pub url: Option<String>,
}

impl UpdateWebCallbacksUrl {
  pub fn run(self, client: &Client) -> ClientResult<AccountState> {
    let fields = gql_fields::ACCOUNT_STATE;
    let method = "updateWebCallbacksUrl";
    client.simple(&self, method, &format!(
      "mutation($url: String) {{ {method}(url: $url) {{ {fields} }} }}",
    ))
  }
}
