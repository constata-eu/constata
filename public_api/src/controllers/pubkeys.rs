use super::*;
use i18n::Lang;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct SignupForm {
  signed_payload: SignedPayload,
}

impl SignupForm {
  pub async fn get_or_create(&self, site: &Site, l: Lang) -> Result<Pubkey> {
    let signer = &self.signed_payload.signer.to_string();

    if let Some(key) = site.pubkey().find_optional(signer).await? {
      return Ok(key);
    }

    let person_id = site.org().insert(Default::default())
      .save_and_subscribe(l).await?.admin().await?.attrs.id;

    site.pubkey().create_from_signed_payload(person_id, &self.signed_payload).await
  }
}

#[post("/", data = "<form>")]
pub async fn create(form: Json<SignupForm>, site: &State<Site>, l: Lang) -> JsonResult<Pubkey> {
  Ok(Json(form.get_or_create(&site, l).await?))
}
