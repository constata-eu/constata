use super::{Serialize, Site};
#[cfg(test)]
pub use constata_lib::test_support::{assert_that, TestDb, SignerClient, rematch};
pub use constata_lib::models::hasher;
use rocket::{
  http::{Header, Status},
  local::asynchronous::{Client, LocalResponse},
};
pub use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct Payload {
  constata_eu_action: String,
  expires: String,
}

#[derive(Deserialize)]
pub struct ApiError {
  pub error: String,
}

#[cfg(test)]
macro_rules! apitest {
  ($i:ident($site:ident, $c:ident, $($client:ident)+) $($e:tt)* ) => {
    test!{ $i
      use crate::test_support::*;
      let $c = TestDb::new().await?;
      let $site = $c.site.clone();
      let $($client)+ = crate::test_support::PublicApiClient::new($c.alice().await).await;
      $($e)*
    }
  };
}

#[cfg(test)]
macro_rules! fulltest {
  ($i:ident($site:ident, $c:ident, $($client:ident)+, $($chain:ident)+) $($e:tt)* ) => {
    test!{ $i
      use crate::test_support::*;
      let $c = TestDb::new().await?;
      let $site = $c.site.clone();
      let $($client)+ = crate::test_support::PublicApiClient::new($c.alice().await).await;
      let $($chain)+ = TestBlockchain::new().await;
      $($e)*
    }
  };
}

pub struct PublicApiClient {
  pub client: Client,
  pub c: TestDb,
  pub signer: SignerClient,
  pub lang: i18n::Lang,
}

impl PublicApiClient {
  pub async fn new(signer: SignerClient) -> Self {
    Self::new_with_lang(signer, i18n::Lang::Es).await
  }

  pub async fn new_with_lang(signer: SignerClient, lang: i18n::Lang) -> Self {
    Self {
      client: Client::tracked(crate::server(Site::default_with_keyring("password").await.unwrap())).await.unwrap(),
      c: signer.db.clone(),
      signer,
      lang
    }
  }

  pub fn set_lang(&mut self, lang: i18n::Lang) {
    self.lang = lang;
  }

  pub fn make_auth_header<'a>(&'a self, path: &str, method: &str, nonce: i64, body: Option<&str>, query: Option<&str>) -> Header<'static> {
    let body_hash = body.map(|x| hasher::hexdigest(x.as_bytes()));
    let query_hash = query.map(|x| hasher::hexdigest(x.as_bytes()));
    let payload = serde_json::json![{
      "path": path,
      "method": method,
      "nonce": nonce,
      "body_hash": body_hash,
      "query_hash": query_hash,
    }].to_string();
    let token = serde_json::to_string(&self.signer.signed_payload(payload.as_bytes()))
      .expect("Signed payload to be serialized");
    Header::new("Authentication", token)
  }

  pub fn make_legacy_auth_header<'a>(&'a self, path: &str, date: &str) -> Header<'static> {
    let payload = serde_json::json![{ "constata_eu_action": path, "expires": date, }].to_string();
    let token = serde_json::to_string(&self.signer.signed_payload(payload.as_bytes()))
      .expect("Signed payload to be serialized");
    Header::new("Authentication", token)
  }

  pub fn ok_auth_header(&self, path: &str, method: &str, body: Option<&str>, query: Option<&str>) -> Header<'static> {
    self.make_auth_header(path , method, chrono::Utc::now().timestamp_millis(), body, query)
  }

  pub async fn post_response<'a, B>(&'a self, path: &'a str, body: B) -> LocalResponse<'a>
  where
    B: AsRef<str> + AsRef<[u8]>,
  {
    let header = self.ok_auth_header(path, "POST", Some(body.as_ref()), None);
    self
      .client
      .post(path)
      .header(header)
      .body(body)
      .dispatch()
      .await
  }

  pub async fn post_string<'a, B>(&'a self, path: &'a str, body: B) -> String
  where
    B: AsRef<str> + AsRef<[u8]>,
  {
    self
      .post_response(path, body)
      .await
      .into_string()
      .await
      .unwrap()
  }

  pub async fn post<'a, T, B>(&'a self, path: &'a str, body: B) -> T
  where
    T: DeserializeOwned,
    B: AsRef<str> + AsRef<[u8]>,
  {
    let string = self.post_string(path, body).await;
    serde_json::from_str(&string).unwrap_or_else(|_| panic!("Could not parse response {}", string))
  }

  pub async fn get<T: DeserializeOwned>(&self, path: &str) -> T {
    let response = self.get_string(path).await;
    serde_json::from_str(&response).expect(&format!("Could not parse response {}", response))
  }

  pub async fn get_string(&self, path: &str) -> String {
    self.get_response(path).await.into_string().await.unwrap()
  }

  pub async fn get_bytes(&self, path: &str) -> Vec<u8> {
    self.get_response(path).await.into_bytes().await.unwrap()
  }

  pub async fn get_response<'a>(&'a self, path: &'a str) -> LocalResponse<'a> {
    self.get_response_with_auth(path, self.ok_auth_header(path, "GET", None, None)).await
  }

  pub async fn get_response_with_auth<'a>(&'a self, path: &'a str, auth_header: Header<'static>) -> LocalResponse<'a> {
    self.client
      .get(path)
      .header(Header::new("Accept-Language", format!("{}-xy", self.lang.code())))
      .header(auth_header)
      .dispatch().await
  }

  pub async fn get_response_with_wrong_auth_path<'a>(&'a self, path: &'a str) -> LocalResponse<'a> {
    self.get_response_with_auth(path, self.ok_auth_header("/boguspath/", "GET", None, None)).await
  }
  
  pub async fn get_response_with_old_auth_token<'a>(&'a self, path: &'a str) -> LocalResponse<'a> {
    self.get_response_with_auth(path, self.make_auth_header(path, "GET", -1, None, None)).await
  }

  pub async fn get_response_with_bad_auth_signature<'a>(&'a self, path: &'a str) -> LocalResponse<'a> {
    let payload = serde_json::json![{
      "path": path,
      "method": "GET",
      "nonce": 1,
      "body_hash": null,
      "query_hash": null,
    }].to_string();
    let token = serde_json::to_string(&self.signer.wrong_signed_payload(payload.as_bytes()))
      .expect("Signed payload to be serialized");

    self.get_response_with_auth(path, Header::new("Authentication", token)).await
  }

  pub async fn get_response_with_malformed_payload<'a>(&'a self, path: &'a str) -> LocalResponse<'a> {
    let payload = serde_json::json![{
      "no_path": path,
      "and_nothing_else": "totally_invalid",
    }]
    .to_string();

    let token =
      serde_json::to_string(&self.signer.signed_payload(payload.as_bytes())).unwrap();

    self.get_response_with_auth(path, Header::new("Authentication", token)).await
  }

  pub async fn assert_unauthorized_get<P: std::fmt::Display>(&self, path: P) {
    let response = self.client.get(path.to_string()).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  pub async fn assert_get_error<'a>(&'a self, path: &'a str, status: Status, msg: &'a str) {
    let response = self.get_response(path).await;
    assert_eq!(response.status(), status);
    let err: ApiError = serde_json::from_str(&response.into_string().await.unwrap()).unwrap();
    assert_that!(&err.error, rematch(msg));
  }
  
}
