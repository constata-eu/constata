use super::*;
use gql_types::issuance_graphql::{
  Issuance,
  CreateIssuanceFromJsonInput,
};

#[derive(serde::Serialize, clap::Args)]
pub struct CreateIssuanceFromJson {
  #[command(flatten)]
  pub input: CreateIssuanceFromJsonInput,

  #[arg(long, help="A path to a PNG or JPEG file to use as logo in your new template")]
  #[serde(skip)]
  pub new_logo_image_file: Option<PathBuf>,

  #[arg(long, help="A path to file with the JSON for your entries, an alternative to --entry")]
  #[serde(skip)]
  pub json_file: Option<PathBuf>,
}

impl CreateIssuanceFromJson {
  pub fn run(mut self, client: &Client) -> ClientResult<Issuance> {
    if let Some(ref path) = self.new_logo_image_file {
      self.input.new_logo_image = Some(ex::fs::read(path)?);
    }

    if let Some(ref path) = self.json_file {
      self.input.entries = check!(
        serde_json::from_str(&ex::fs::read_to_string(path)?),
        InvalidInput("your --json-file could not be parsed")
      );
    }

    client.mutation(&self, "createIssuanceFromJson", "CreateIssuanceFromJsonInput", gql_fields::ISSUANCE)
  }
}
