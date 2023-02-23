use validators::traits::ValidateString;

pub use constata_lib::{
  bitcoin::Address,
  models::{*, document::Document},
  serde::{self, Deserialize, Serialize, de::DeserializeOwned},
  signed_payload::SignedPayload,
};

pub use rocket::{
  self,
  fairing::AdHoc,
  form::{self, FromForm, Form, Strict},
  get,
  http::{uri::Path, Status, Header, RawStr},
  response::Responder,
  post,
  request::{FromRequest, Outcome, Request},
  routes,
  serde::json::Json,
  State,
  data::{self, Data, FromData, ToByteUnit, Limits},
};
use juniper::http::{GraphQLBatchRequest };
use juniper_rocket::GraphQLRequest;

#[derive(Debug, PartialEq)]
pub enum AuthMethod {
  Legacy{ meta: LegacyAuth, pubkey: Pubkey, auth_string: String },
  Signed{ meta: ApiRequestMetadata, pubkey: Pubkey, auth_string: String },
  Token{ token: AccessToken },
  Forced,
}

#[derive(Debug, PartialEq)]
pub struct CurrentPerson{
  pub person: Person,
  pub method: AuthMethod,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ApiRequestMetadata {
  pub path: String,
  pub method: String,
  pub nonce: i64,
  pub body_hash: Option<String>,
  pub query_hash: Option<String>,
}

// This LegacyAuth is deprecated.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct LegacyAuth {
  pub constata_eu_action: String,
  pub expires: UtcDateTime,
}

impl CurrentPerson {
  pub fn evidence(&self) -> &[u8] {
    match &self.method {
      AuthMethod::Legacy{ auth_string, .. } => auth_string.as_bytes(),
      AuthMethod::Signed{ auth_string, .. } => auth_string.as_bytes(),
      AuthMethod::Token{..} => b"{}",
      AuthMethod::Forced{..} => b"{}",
    }
  }

  async fn build(req: &Request<'_>, body: Option<&[u8]>) -> Option<Self> {
    let site = req.rocket().state::<Site>()?;

    if let Some(access_token) = req.headers().get_one("Access-Token") {
      let token = site.access_token().find_active(access_token.to_string()).one().await.ok()?;
      let person = token.person().await.ok()?;

      if person.deletion_id().is_none() && token.allows(&Self::operation_name(req, &body)?) {
        return Some(Self{ person, method: AuthMethod::Token{ token }});
      }

      return None;
    }

    let auth_string = req.headers().get_one("Authentication")?.to_string();
    let signed_payload = serde_json::from_str::<SignedPayload>(&auth_string).ok()?;
    signed_payload.signed_ok().ok()?.then(|| ())?;

    let utf8 = std::str::from_utf8(&signed_payload.payload).ok()?;
    if let Ok(meta) = serde_json::from_str::<ApiRequestMetadata>(utf8) {
      (meta.path == req.uri().path().raw()).then(|| ())?;
      (meta.method == req.method().as_str()).then(|| ())?;

      if let Some(ref query_hash) = meta.query_hash {
        (hex::decode(query_hash).ok()? == hasher::digest(req.uri().query()?.raw().as_bytes())).then(|| ())?;
      }

      if let Some(ref body_hash) = meta.body_hash {
        (hex::decode(body_hash).ok()? == hasher::digest(body?)).then(|| ())?;
      }

      let maybe_pubkey = site.pubkey().select()
        .id_eq(&signed_payload.signer.to_string())
        .one().await.ok();

      let (person, pubkey) = match maybe_pubkey {
        Some(pubkey) => {
          if pubkey.deletion_id().is_some() || meta.nonce < pubkey.attrs.nonce as i64 {
            return None;
          }
          let p = pubkey.person().await;
          let u = pubkey.update().nonce(meta.nonce as i64).save().await;
          (p.ok()?, u.ok()?)
        },
        None => {
          let signup_token = req.headers().get_one("Signup-Token")?;
          let token = rocket_recaptcha_v3::ReCaptchaToken::parse_str(signup_token).ok()?;
          let encrypted_key = req.headers().get_one("Signup-Encrypted-Key")?;
          //let remote_ip = req.headers().get_one("CF-Connecting-IP");
          let lang = req.guard::<i18n::Lang>().await.succeeded()?;
          let recaptcha = req.guard::<&rocket::State<rocket_recaptcha_v3::ReCaptcha>>().await.succeeded()?;
          let public_key = signed_payload.pubkey().ok()?.to_string();

          let verification = recaptcha.verify(&token, None).await.ok()?;

          if verification.score < site.settings.recaptcha_threshold() {
            return None;
          }

          let tx = site.org().transactional().await.ok()?;
          let person = tx.insert(Default::default())
            .save_and_subscribe(lang).await.ok()?.admin().await.ok()?;

          let pubkey = person.state.pubkey().create_from_signed_payload_with_backup(
            person.attrs.id,
            &signed_payload,
            encrypted_key,
            &public_key,
          ).await.ok()?;

          tx.commit().await.ok()?;
          (person, pubkey)
        }
      };
      return Some(Self{ person, method: AuthMethod::Signed{ meta, pubkey, auth_string }});

    } else if let Ok(meta) = serde_json::from_str::<LegacyAuth>(utf8) {
      (meta.constata_eu_action == req.uri().path().raw()).then(|| ())?;
      (meta.expires > chrono::offset::Utc::now()).then(|| ())?;
      let pubkey = site.pubkey().select()
        .id_eq(&signed_payload.signer.to_string())
        .deletion_id_is_set(false)
        .one().await.ok()?;

      return Some(Self{ person: pubkey.person().await.ok()?, method: AuthMethod::Legacy{ meta, pubkey, auth_string }});
    };

    return None;
  }

  pub fn operation_name(req: &Request, body: &Option<&[u8]>) -> Option<String> {
    match body {
      Some(b) => {
        let r = serde_json::from_str::<GraphQLBatchRequest>(&String::from_utf8_lossy(&b)).ok()?;
        GraphQLRequest(r).operation_names().pop()?.map(|n| n.to_string())
      },
      None => {
        let q = req.uri().query()?;
        Form::<Strict<GraphQLRequest>>::parse_encoded(q.raw()).ok()?.operation_names().pop()?.map(|n| n.to_string())
      }
    }
  }

}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CurrentPerson {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    match CurrentPerson::build(req, None).await {
      Some(current) => Outcome::Success(current),
      None => Outcome::Failure((Status::Unauthorized, ())),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct CurrentPersonAndJson<T>{
  pub person: CurrentPerson,
  pub json: T,
}

#[rocket::async_trait]
impl<'r, T: DeserializeOwned> FromData<'r> for CurrentPersonAndJson<T> {
  type Error = ();

  async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
    use rocket::data::Outcome;
    let limit = req.limits().get("json").unwrap_or(Limits::JSON);

    let body_bytes = match data.open(limit).into_bytes().await {
      Ok(read) if read.is_complete() => read.into_inner(),
      _ => return Outcome::Failure((Status::BadRequest, ())),
    };
    
    match CurrentPerson::build(req, Some(&body_bytes)).await {
      Some(current) => {
        match serde_json::from_str(&String::from_utf8_lossy(&body_bytes)) {
          Ok(value) => Outcome::Success(CurrentPersonAndJson{
            person: current,
            json: value
          }),
          Err(_) => Outcome::Failure((Status::BadRequest, ())),
        }
      },
      None => Outcome::Failure((Status::Unauthorized, ())),
    }
  }
}
