use super::*;
use models::download_proof_link;

#[derive(GraphQLObject)]
#[graphql(description = "This resource is used by Constata's admin front-end to show options for viewing, downolading or sharing a certificate on social networks. NOTICE: You should probably never use this resource yourself, use the Attestation and Issuance resources instead. If you insist on using it, keep in mind this endpoint does not authenticate using the 'Authentication' header, you should send an Auth-Token header with the special token generated for administrative access.")]
pub struct DownloadProofLink {
  #[graphql(description = "Unique ID for this link.")]
  pub id: i32,
  #[graphql(description = "Expiration date, this admin link will not be valid after this date and time.")]
  pub valid_until: Option<UtcDateTime>,
  #[graphql(description = "A verifiable certificate may contain several documents. This many are pending to be certified.")]
  pub pending_doc_count: i32,
  #[graphql(description = "Creation date of the last document in this verifiable certificate.")]
  pub last_doc_date: Option<UtcDateTime>,
  #[graphql(description = "Public URL that displays this certificate, if sharing is active.")]
  pub public_certificate_url: String,
  #[graphql(description = "Whether this certificate's sharing is active. If so, the public_certificate_url can be shared on social media and viewed by anyone.")]
  pub public_certificate_is_active: bool,
  #[graphql(description = "The text to display in the front-end for the 'share on social media' call to action.")]
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
#[graphql(description = "This input object allows changing the public_certificate_is_active flag, to enable and disable sharing publicly.")]
pub struct DownloadProofLinkInput {
  #[graphql(description = "Send 'publish' to enable, or 'unpublish' to disable. Any other value will be understood as 'unpublish'.")]
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
