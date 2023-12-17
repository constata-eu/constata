use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "A link to download a document")]
pub struct DownloadProofLink {
  id: String,
  url: Option<String>,
}

impl DownloadProofLink {
  pub async fn create_link(context: &Context, document_id: String) -> FieldResult<DownloadProofLink> {
    if context.role != AdminRole::SuperAdmin {
      return Err(field_error("401", "you don't have permission and you tried to hack the UI"));
    }

    Ok(DownloadProofLink { id: document_id, url: None })
  }
}
