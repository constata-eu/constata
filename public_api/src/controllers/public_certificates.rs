use super::*;
use constata_lib::models::blockchain::PrivateKey;
use serde_json::json;
use crate::RENDERER;

#[get("/<token>?<show_content>")]
pub async fn show(site: &State<Site>, token: String, key: &State<PrivateKey>, l: Lang, show_content: bool) -> ConstataResult<i18n::LocalizedResponse<'static>> {
  let response = match site.download_proof_link().public_certificate_active(token).one().await {
    Err(_e) => RENDERER.i18n("certificates/", l, "not_found.html")?,
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
        RENDERER.i18n_and_serialize("certificates/", l, "to_show.html", &context)?
      }
    }
  };
  Ok(response)
}
