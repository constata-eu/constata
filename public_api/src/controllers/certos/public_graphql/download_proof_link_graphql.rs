use super::*;
use models::download_proof_link;

#[derive(GraphQLObject)]
#[graphql(description = "This object is an adminitrative link that allows the receiver of the certificate to see, download or publish the certificate")]
pub struct DownloadProofLink {
  #[graphql(description = "number identifying a Download Proof Link")]
  pub id: i32,
  #[graphql(description = "expiration date if any")]
  pub valid_until: Option<UtcDateTime>,
  #[graphql(description = "amount of pending documents for certification in the same story")]
  pub pending_doc_count: i32,
  #[graphql(description = "date in which was created the last document if are more than one")]
  pub last_doc_date: Option<UtcDateTime>,
  #[graphql(description = "url to see the public certificate")]
  pub public_certificate_url: String,
  #[graphql(description = "boolean pointing out if the public certificate is active")]
  pub public_certificate_is_active: bool,
  #[graphql(description = "text to share on social networks along with the public certificate url")]
  pub share_on_social_networks_call_to_action: String,
}

impl DownloadProofLink {
  pub async fn from_context(context: &Context) -> FieldResult<download_proof_link::DownloadProofLink> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      Ok(context.site.download_proof_link()
        .active(token.attrs.token.clone()).one().await?)
    } else {
      Err(field_error("access", "invalid_download_proof_link"))
    }
  }

  pub async fn from_db(db_download_proof_link: &download_proof_link::DownloadProofLink, l: &i18n::Lang) -> FieldResult<DownloadProofLink> {
    let pending_docs = db_download_proof_link.document().await?.story().await?.pending_docs().await?;
    
    Ok(DownloadProofLink{
      id: db_download_proof_link.attrs.id,
      valid_until: db_download_proof_link.valid_until().await?,
      pending_doc_count: pending_docs.len() as i32,
      last_doc_date: pending_docs.last().map(|d| d.attrs.created_at.clone()),
      public_certificate_url: db_download_proof_link.public_certificate_url(),
      public_certificate_is_active: db_download_proof_link.published_at().is_some(),
      share_on_social_networks_call_to_action: db_download_proof_link.share_on_social_networks_call_to_action(l).await?,
    })
  }

  pub async fn download_proof_link(context: &Context) -> FieldResult<DownloadProofLink> {
    let download_proof_link = DownloadProofLink::from_context(context).await?;
    DownloadProofLink::from_db(&download_proof_link, &context.lang).await
  }

  pub async fn delete_download_proof_link(context: &Context) -> FieldResult<DownloadProofLink> {
    let download_proof_link = DownloadProofLink::from_context(context).await?;
    download_proof_link.access_token().await?.expire().await?;
    DownloadProofLink::from_db(&download_proof_link.reloaded().await?, &context.lang).await
  }
}


#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "This object allows to change the state of the public certificate")]
pub struct DownloadProofLinkInput {
  #[graphql(description = "the actions to use are 'publish' and 'unpublish'")]
  pub action: String,
}

impl DownloadProofLinkInput {
  pub async fn update_download_proof_link(&self, context: &Context) -> FieldResult<DownloadProofLink> {
    let download_proof_link = DownloadProofLink::from_context(context).await?;
    if self.action == "publish" {
      download_proof_link.publish().await?;
    } else {
      download_proof_link.unpublish().await?;
    }
    DownloadProofLink::from_db(&download_proof_link.reloaded().await?, &context.lang).await
  }
}
