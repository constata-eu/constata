use super::*;
use gql_types::{
  attestation_graphql::{
    Attestation,
    AttestationInput,
  }
};

#[derive(serde::Serialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttestation {
  /// A list of paths to the files to add to your attestation.
  #[arg(short, long="path", value_name="PATH", action=clap::ArgAction::Append,
    help="Path to a file you want to include in your attestation. You can repeat this argument to add many.")]
  pub paths: Vec<PathBuf>,

  /// An attestation allows appending documents up until a certain date. If you don't chose a date, no appending will be allowed.
  #[arg(short)]
  pub open_until: Option<constata_lib::models::UtcDateTime>,

  /// Markers is a text that can be used for searching this attestation later. Markers cannot be updated after creation.
  #[arg(short)]
  pub markers: Option<String>,

  /// A list of email addresses to notify when the documents are attested.
  /// Constata will email them an administrative access link to view, download or share the document certificate.
  /// You can pass an empty list if you want to omit Constata's emails, and manage distribution of the attestation on your own.
  #[arg(short, long, value_name="PATH", action=clap::ArgAction::Append,
    help="Email an admin access link to view, download and share this attestation to the given addresses.")]
  pub email_admin_access_url_to: Vec<String>,
}

impl CreateAttestation {
  pub fn run(self, client: &Client) -> ClientResult<Attestation> {
    let documents: Vec<SignedPayload> = self.paths.iter()
      .map(|f| ex::fs::read(f).map(|x| client.sign(&x) ).into() )
      .collect::<Result<Vec<_>, ex::io::Error>>()?;

    QueryWithAttestationInput::from(AttestationInput{
      documents,
      open_until: self.open_until,
      markers: self.markers,
      email_admin_access_url_to: self.email_admin_access_url_to,
    }).run(client)
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QueryWithAttestationInput {
  input: AttestationInput,
}

impl From<AttestationInput> for QueryWithAttestationInput {
  fn from(input: AttestationInput) -> Self {
    QueryWithAttestationInput{ input }
  }
}

impl QueryWithAttestationInput {
  pub fn run(&self, client: &Client) -> ClientResult<Attestation> {
    client.mutation(self, "createAttestation", "AttestationInput", gql_fields::ATTESTATION)
  }
}
