use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "This object retrieves a certificate")]
pub struct Proof {
  #[graphql(description = "number identifying the proof")]
  pub id: i32,
  #[graphql(description = "certificate in html format")]
  pub html: String,
}

impl Proof {
  pub async fn proof(context: &Context) -> FieldResult<Proof> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      let download_proof_link = context.site.download_proof_link()
        .active(token.attrs.token.clone()).one().await?;

      Ok(Proof{
        id: download_proof_link.attrs.id,
        html: download_proof_link.html_proof(&context.key, context.lang).await?,
      })
    } else {
      Err(field_error("access", "invalid_download_proof_link"))
    }
  }
}