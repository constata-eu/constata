use super::*;
use gql_types::*;

#[derive(serde::Serialize, clap::Args)]
pub struct CreateIssuanceFromCsv {
  #[command(flatten)]
  pub input: CreateIssuanceFromCsvInput,

  #[arg(long, help="A path to a PNG or JPEG file to use as logo in your new template")]
  #[serde(skip)]
  pub new_logo_image_file: Option<PathBuf>,

  #[arg(long, help="A path to a CSV file you want to use for creating this issuance.")]
  #[serde(skip)]
  pub csv_file: Option<PathBuf>,
}

impl CreateIssuanceFromCsv {
  pub fn run(mut self, client: &Client) -> ClientResult<Issuance> {
    if let Some(ref path) = self.new_logo_image_file {
      self.input.new_logo_image = Some(ex::fs::read(path)?);
    }

    if let Some(ref path) = self.csv_file {
      self.input.csv = ex::fs::read_to_string(path)?;
    }

    client.mutation(&self, "createIssuanceFromCsv", "CreateIssuanceFromCsvInput", gql_fields::ISSUANCE)
  }
}
