use super::*;
use constata_lib::models::blockchain::PrivateKey;
use serde_json::json;


#[get("/<token>?<show_content>")]
pub async fn show(site: &State<Site>, token: String, key: &State<PrivateKey>, l: Lang, show_content: bool) -> Result<i18n::HtmlWithLocale> {
  let response = match site.download_proof_link().public_certificate_active(token).one().await {
    Err(_) => l.html_bare("public_api/certificates/not_found.html.tera")?,
    Ok(download_proof_link) => {
      if show_content {
        i18n::HtmlWithLocale{ lang: l, content: download_proof_link.html_proof(key, l).await?}
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
        l.html_from_serialize("public_api/certificates/to_show.html.tera", &context)?
      }
    }
  };
  Ok(response)
}


