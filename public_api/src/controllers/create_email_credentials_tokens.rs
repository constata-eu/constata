use super::{Result as MyResult, *};
use serde_json::json;
use crate::terms_acceptance::RequestMeta;


#[get("/<token>")]
pub async fn new(token: String, site: &State<Site>, l: Lang) -> MyResult<HtmlWithLocale> {
  let credentials_token = site.create_email_credentials_token().find(&token).await?;
  if *credentials_token.completed() {
    return Ok(l.html_bare("public_api/for_login/create_email_credentials_token/created.html.tera")?)
  }

  let email_address = credentials_token.person().await?.last_email_address().await?.attrs.address;
  Ok(l.html_from_serialize(
    "public_api/for_login/create_email_credentials_token/for_create.html.tera",
    &json!({"email_address": email_address})
  )?)
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct UpdateForm {
  signed_payload: SignedPayload,
}

#[post("/<token>", data="<form>")]
pub async fn update(site: &State<Site>, token: String, form: Json<UpdateForm>, meta: RequestMeta) -> JsonResult<Pubkey> {
  let credentials_token = site.create_email_credentials_token().select().id_eq(&token).one().await?;
  let pubkey = credentials_token.link_pubkey(&form.signed_payload, meta.0.as_bytes()).await?;
  Ok(Json(pubkey))
}


constata_lib::describe_one! {
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  struct ApiPubkey {
    id: String,
    person_id: i32,
  }

  test!{ can_update_from_link
    let c = TestDb::new().await?;
    let site = c.site.clone();
    let eve = c.eve().await;
    let client = crate::test_support::PublicApiClient::new(eve.clone()).await;

    let bot = c.bot().await.accept_terms_and_conditions().await;
    let person = site.person().find(&bot.person_id).await?;
    person.get_or_create_email_credentials_token_url().await?;
    let credentials_token = &person.create_email_credentials_token_vec().await?[0];

    let url = format!("/create_email_credentials_token/{}", credentials_token.id());
    let page = client.get_string(&url).await;
    let email_address = &person.email_address_vec().await?[0];
    let esta_por_vincular = format!(
      "Está por vincular su email {} a su nueva clave privada.",
      email_address.address()
    );
    assert_that!(&page, rematch(&esta_por_vincular));
    let payload = format!("{}-{}", email_address.address(), credentials_token.id());

    let pubkey: ApiPubkey = client.post(&url,
      serde_json::json![{
        "signed_payload": eve.signed_payload(payload.as_bytes()),
        "token": credentials_token.id(),
      }].to_string()
    ).await;
    
    assert_eq!(pubkey.person_id, bot.person_id);
    assert_eq!(credentials_token.reloaded().await?.attrs.completed, true);

    let page_already_used = client.get_string(&url).await;
    assert_that!(&page_already_used, rematch("Este link ya fue utilizado para crear credenciales y no puede ser utilizado nuevamente."));
    assert_that!(&page_already_used, not(rematch(&esta_por_vincular)));
  }
}
