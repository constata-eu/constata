use super::*;

#[derive(serde::Serialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct AllAttestationsHtmlExport {
  #[arg(help="Save the verifiable HTMLs to the given directory if possible. \
    Will fail if it encounters an attestation that has no verifiable html available yet.")]
  pub path: PathBuf,

  #[arg(short, long, help="Do not fail if we encounter an attestation with no verifiable HTML, skip it instead.")]
  pub dont_fail_on_missing: bool,

  #[command(flatten)]
  pub all_attestations: AllAttestations,
}

impl AllAttestationsHtmlExport {
  pub fn run<F: Fn(i32, i32, &gql_types::Attestation)>(&self, client: &Client, before_each_save: F) -> ClientResult<i32> {
    if !self.path.is_dir() {
      return Err(Error::NotFound(format!("a directory called {}", &self.path.display())))
    }

    let output = self.all_attestations.run(client)?;
    let total = output.meta.count;
    let mut current = 1;
    let mut saved = 0;

    for attestation in &output.all {
      before_each_save(current, total, attestation);

      let exported = AttestationHtmlExport {
        id: attestation.id, 
        out_file: Some(self.path.join(format!("attestation_{}.html", attestation.id))),
      }.run(client);
      current += 1;

      match exported {
        Ok(_) => saved += 1,
        Err(e) => if !self.dont_fail_on_missing { return Err(e) }
      }
    }

    Ok(saved)
  }
}
