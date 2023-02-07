use super::*;
#[cfg(test)]
pub use constata_lib::test_support::*;
use constata_lib::models::admin_user::AdminRole;

use rocket::{
  http::Header,
  local::asynchronous::{Client, LocalResponse},
};
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
use google_authenticator::GoogleAuthenticator;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct Payload {
  constata_eu_action: String,
  expires: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphqlError {
  errors: Vec<GraphqlMessage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphqlMessage {
  message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphqlDataAll {
  data: GraphqlAll,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlAll {
  pub all_payments: Option<Vec<GraphqlAllIds>>,
  pub all_bulletins: Option<Vec<GraphqlAllIds>>,
  pub all_documents: Option<Vec<GraphqlStringId>>,
  pub all_people: Option<Vec<GraphqlAllIds>>,
  pub all_invoices: Option<Vec<GraphqlAllIds>>,
  pub all_invoice_links: Option<Vec<GraphqlAllIds>>,
  pub all_subscriptions: Option<Vec<GraphqlAllIds>>,
  pub all_email_addresses: Option<Vec<GraphqlAllIds>>,
  pub all_gifts: Option<Vec<GraphqlAllIds>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlAllIds {
  id: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlStringId {
  id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlDataMeta {
  data: GraphqlMeta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlMeta {
  #[serde(rename="_allPaymentsMeta")]
  pub _all_payments_meta: Option<Count>,
  #[serde(rename="_allBulletinsMeta")]
  pub _all_bulletins_meta: Option<Count>,
  #[serde(rename="_allDocumentsMeta")]
  pub _all_documents_meta: Option<Count>,
  #[serde(rename="_allPeopleMeta")]
  pub _all_people_meta: Option<Count>,
  #[serde(rename="_allInvoicesMeta")]
  pub _all_invoices_meta: Option<Count>,
  #[serde(rename="_allInvoiceLinksMeta")]
  pub _all_invoice_links_meta: Option<Count>,
  #[serde(rename="_allSubscriptionsMeta")]
  pub _all_subscriptions_meta: Option<Count>,
  #[serde(rename="_allEmailAddressesMeta")]
  pub _all_email_addresses_meta: Option<Count>,
  #[serde(rename="_allGiftsMeta")]
  pub _all_gifts_meta: Option<Count>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Count {
  pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTest {
  token: String,
}

#[derive(Deserialize)]
pub struct ApiError {
  pub error: String,
}

#[cfg(test)]
macro_rules! apitest {
  ($i:ident($site:ident, $c:ident, $client:ident) $($e:tt)* ) => {
    test!{ $i
      let $c = TestDb::new().await?;
      let $site = $c.site.clone();
      let $client = crate::test_support::PrivateApiClient::new(crate::server($c.site.clone()), $c.clone()).await;
      $($e)*
    }
  };
}

pub struct PrivateApiClient {
  pub client: Client,
  pub c: TestDb,
}

impl PrivateApiClient {
  pub async fn new(server: rocket::Rocket<rocket::Build>, c: TestDb) -> Self {
    Self {
      client: Client::tracked(server).await.unwrap(),
      c,
    }
  }

  pub async fn post_for_session<'a>(&'a self, path: &'a str, body: String) -> LocalResponse<'a> {
    self
      .client
      .post(path)
      .body(body)
      .dispatch()
      .await
  }

  pub async fn post_for_session_and_get_token(& self, path: &str, body: String) -> String {
    let response = self.post_for_session(path, body).await
                       .into_string().await.expect("Fetching response");
    let session: SessionTest = serde_json::from_str(response.as_str()).expect("A session test");
    session.token
  }

  pub async fn post_with_token_raw<'a>(&'a self, path: &'a str, auth_token: String, body: String) -> LocalResponse<'a> {
    self
      .client
      .post(path)
      .body(body)
      .header(Header::new("Content-Type", "application/graphql"))
      .header(Header::new("Authorization", auth_token))
      .dispatch()
      .await
  }

  pub async fn post_with_token(&self, path: &str, auth_token: String, body: String) -> String {
    let response = self.post_with_token_raw(path, auth_token, body).await;
    response.into_string().await.expect("post_with_token to have a string body")
  }

  pub async fn post_with_no_token<'a>(&'a self, path: &'a str, body: String) -> LocalResponse<'a> {
    self
      .client
      .post(path)
      .body(body)
      .header(Header::new("Content-Type", "application/graphql"))
      .dispatch()
      .await
  }

  pub async fn post_with_token_message_graphql(&self, path: &str, auth_token: String, body: String) -> String {
    let response = self.post_with_token(path, auth_token, body).await;
    let graphql_error: GraphqlError = serde_json::from_str(response.as_str()).unwrap();
    graphql_error.errors[0].message.clone()
  }

  pub async fn post_for_all_search(&self, path: &str, auth_token: String, body: String) -> GraphqlAll {
    let response = self.post_with_token(path, auth_token, body).await;
    let graphql_data: GraphqlDataAll = serde_json::from_str(response.as_str()).unwrap();
    graphql_data.data.clone()
  }

  pub async fn post_for_meta_search(&self, path: &str, auth_token: String, body: String) -> GraphqlMeta {
    let response = self.post_with_token(path, auth_token, body).await;
    let graphql_data: GraphqlDataMeta = serde_json::from_str(response.as_str()).unwrap();
    graphql_data.data.clone()
  }

  pub async fn login_and_get_token(&self, username: &str, password: &str) -> String {
    self.c.site.admin_user()
      .create(username, password, AdminRole::Admin)
      .await
      .expect("to be able to create an admin");

    let admin = self.c.site.admin_user()
      .find_from_credentials(username, password)
      .await.expect("Admin user to exist");

    let otp = GoogleAuthenticator::new().get_code(&admin.attrs.otp_seed, 0).unwrap();

    let body = serde_json::json![{
      "username": username,
      "password": password,
      "otp": &otp
    }].to_string();
    
    self.post_for_session_and_get_token("/sessions/", body).await
  }
}
