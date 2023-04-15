use super::*;

#[derive(serde::Serialize, clap::Args)]
pub struct CustomGraphql {
  /// The variables for your query as a JSON string '{"foo": "bar"}'
  #[arg(value_parser=variables_parser)]
  variables: serde_json::Value,

  /// Your full query, check our graphiql explorer to build your own queries.
  query: String,
}

fn variables_parser(s: &str) -> Result<serde_json::Value, String> {
  serde_json::from_str(&s).map_err(|e| format!("Variables are not valid json: {:#?}", e) )
}

impl CustomGraphql {
  pub fn run(&self, client: &Client) -> ClientResult<serde_json::Value> {
    client.query(&self.variables, &self.query)
  }
}
