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

    let document = context.site.document().find(&document_id).await?;
    let mut url = None;

    if let Ok(accepted) = document.in_accepted() {
      if accepted.bulletin().await?.is_published() {
        url = Some(document.create_download_proof_link(7).await?.full_url().await?);
      }
    }

    Ok(DownloadProofLink { id: document_id, url, })
  }
}
