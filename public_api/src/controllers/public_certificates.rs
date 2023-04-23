use super::*;
use constata_lib::models::blockchain::PrivateKey;
use serde_json::json;
use crate::RENDERER;

#[get("/<token>?<show_content>")]
pub async fn show(site: &State<Site>, token: String, key: &State<PrivateKey>, l: Lang, show_content: bool) -> Result<i18n::LocalizedResponse<'static>> {
  let response = match site.download_proof_link().public_certificate_active(token).one().await {
    Err(_e) => RENDERER.render_localized("public_api/certificates/", &std::path::PathBuf::from("not_found.html"), l, Lang::En)?,
    Ok(download_proof_link) => {
      if show_content {
        i18n::LocalizedResponse::new(
          std::borrow::Cow::Owned(download_proof_link.html_proof(key, l).await?.into_bytes()),
          rocket::http::ContentType::HTML,
          l
        )
      } else {
        download_proof_link.update_public_visit_count().await?;
        let org = download_proof_link.org().await?;
        let context = json!({
          "title": download_proof_link.title().await?,
          "template_kind": download_proof_link.template_kind().await?,
          "public_name": org.attrs.public_name,
          "name_for_on_behalf_of": org.name_for_on_behalf_of().await?,
          "image": download_proof_link.image_url().await?,
        });
        RENDERER.render_localized_and_serialized("public_api/certificates/", &std::path::PathBuf::from("to_show.html"), l, Lang::En, &context)?
      }
    }
  };
  Ok(response)
}
