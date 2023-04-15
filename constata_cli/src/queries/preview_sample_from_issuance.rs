use super::*;
use gql_types::entry_graphql::PreviewEntry as Model;

#[derive(clap::Args)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSampleFromIssuance {
  #[arg(help="id of the entry you want preview")]
  pub issuance_id: i32,

  #[arg(help="Write the HTML file here, you can then open it with your web browser. \
    Use --json-pointer=html to extract the HTML and print it to stdout.")]
  #[serde(skip)]
  pub out_file: Option<PathBuf>,
}

impl PreviewSampleFromIssuance {
  pub fn run(self, client: &Client) -> ClientResult<Model> {
    use crate::gql_types::EntryFilter;
    let entries = AllEntries{
      filter: EntryFilter{ issuance_id_eq: Some(self.issuance_id), ..Default::default() },
      ..Default::default()
    }.run(&client)?;

    let id = entries.all.get(0)
      .ok_or_else(|| Error::NotFound(format!("an entry for issue {}", &self.issuance_id)))?
      .id;

    PreviewEntry{ id: id, out_file: self.out_file }.run(client)
  }
}
