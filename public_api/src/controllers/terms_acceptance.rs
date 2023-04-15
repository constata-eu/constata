use super::{Result as MyResult, *};
use constata_lib::models::terms_acceptance::TermsAcceptance;
use rocket::data::{Data, ToByteUnit};

#[get("/<token>")]
pub async fn show(token: Option<String>, site: &State<Site>, l: Lang) -> MyResult<HtmlWithLocale> {
  if let Some(t) = token {
    let needed = site.terms_acceptance()
      .select().token_eq(&t).optional().await?
      .map(|a| a.is_needed())
      .unwrap_or(false);
    
    if needed {
      return Ok(l.html_bare("public_api/terms_acceptance/for_acceptance.html.tera")?)
    }
  }

  return Ok(l.html_bare("public_api/terms_acceptance/for_display.html.tera")?)
}

#[get("/")]
pub async fn show_bare(l: Lang) -> MyResult<HtmlWithLocale> {
  return Ok(l.html_bare("public_api/terms_acceptance/for_display.html.tera")?)
}

#[post("/<token>/accept", data="<data>")]
pub async fn accept(token: String, site: &State<Site>, meta: RequestMeta, data: Data<'_>) -> JsonResult<TermsAcceptance> {
  let body = data.open(1.mebibytes()).into_string().await?;
  let headers_and_body = format!("{}\n\n{}", meta.0, body.into_inner());
  
  let tyc = site.terms_acceptance()
    .select()
    .token_eq(&token)
    .one()
    .await?
    .accept(headers_and_body.as_bytes())
    .await?;

  Ok(Json(tyc))
}

#[derive(Debug, Clone)]
pub struct RequestMeta(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestMeta {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    Outcome::Success(RequestMeta(format!("{req:?}")))
  }
}

constata_lib::describe_one! {
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  struct ApiTermsAcceptance { token: String }

  fulltest!{ can_accept_from_link (_site, c, client, _chain)
    let person = c.make_person().await;
    let tyc = person.clone().get_or_create_terms_acceptance().await?;
    assert_eq!(tyc.is_needed(), true);
    let url = format!("/terms_acceptance/{}", tyc.attrs.token);

    let page = client.get_string(&url.clone()).await;
    assert_that!(&page, rematch("CONDICIONES GENERALES DE CONTRATACIÓN"));
    assert_that!(&page, rematch("Confirmo que he leído y acepto los Términos y Condiciones de uso"));

    let url_to_accept = format!("{}/accept", url.clone());
    let _page: ApiTermsAcceptance = client.post(url_to_accept.as_str(), 
      serde_json::json![{ "token": tyc.attrs.token }].to_string()
    ).await;

    let tyc_accepted = person.get_or_create_terms_acceptance().await?;
    assert_eq!(tyc_accepted.is_needed(), false);

    let evidence = String::from_utf8(tyc_accepted.attrs.evidence.unwrap())?;

    assert_that!(&evidence, rematch(r#"Request \{ method: Atomic\(Post\)"#));
    assert_that!(&evidence, rematch("/terms_acceptance/"));

    let page_accepted = client.get_string(&url).await;
    assert_that!(&page_accepted, not(rematch("Confirmo que he leído y acepto los Términos y Condiciones de uso")));
  }

  fulltest!{ has_english_version_for_display (_site, c, _client, _chain)
    let client = crate::test_support::PublicApiClient::new_with_lang(c.bob().await, i18n::Lang::En).await;
    let page = client.get_string("/terms_acceptance").await;
    assert_that!(&page, rematch("GENERAL CONTRACTING CONDITIONS"));
  }
}
