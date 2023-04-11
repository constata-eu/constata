use super::*;
use gql_types::issuance_graphql::{
  Issuance,
  AppendEntriesToIssuanceInput
};

#[derive(serde::Serialize, clap::Args)]
pub struct AppendEntriesToIssuance {
  #[command(flatten)]
  pub input: AppendEntriesToIssuanceInput,

  #[arg(long, help="A path to file with the JSON for your entries, an alternative to --entry")]
  #[serde(skip)]
  pub json_file: Option<PathBuf>,
}

impl AppendEntriesToIssuance {
  pub fn run(mut self, client: &Client) -> ClientResult<Issuance> {
    if let Some(ref path) = self.json_file {
      self.input.entries = check!(
        serde_json::from_str(&ex::fs::read_to_string(path)?),
        InvalidInput("your --json-file could not be parsed")
      );
    }

    client.mutation(&self, "appendEntriesToIssuance", "AppendEntriesToIssuanceInput", gql_fields::ISSUANCE)
  }
}
