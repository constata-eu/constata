use super::*;
use gql_types::{
  attestation_graphql::{
    Attestation,
    AttestationSetPublishedInput,
  }
};

#[derive(serde::Serialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct AttestationSetPublished {
  #[command(flatten)]
  pub input: AttestationSetPublishedInput,
}

impl AttestationSetPublished {
  pub fn run(self, client: &Client) -> ClientResult<Attestation> {
    client.mutation(&self, "attestationSetPublished", "AttestationSetPublishedInput", gql_fields::ATTESTATION)
  }
}
