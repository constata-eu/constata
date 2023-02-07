use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "A Proof")]
pub struct Proof {
  pub id: i32,
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