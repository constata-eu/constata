use super::*;
use constata_lib::models::*;
pub use constata_lib::bitcoin::Address;
pub use i18n::Lang;

pub use rocket::{
  fairing::AdHoc,
  form::{self, FromForm},
  get,
  http::{uri::Path, Status, Header},
  response,
  post,
  request::{FromRequest, Outcome, Request},
  routes,
  serde::json::Json,
  State,
  data::{self, Data, FromData, ToByteUnit},
};

pub mod static_files;
pub mod explorer;
pub mod payments;
pub mod terms_acceptance;
pub mod public_certificates;
pub mod react_app;

constata_lib::describe_one! {
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  struct ApiStory { id: i32 }

  apitest!{ checks_if_signed_payload_is_signed_correctly (_db, _c, alice_client)
    use gql::{AccountState, account_state::Variables};

    let payload = serde_json::json![{
      "path": "/graphql/",
      "method": "POST",
      "nonce": 1,
      "body_hash": null,
      "query_hash": null,
    }].to_string();

    let token = serde_json::to_string(&alice_client.signer.wrong_signed_payload(payload.as_bytes()))
      .expect("Signed payload to be serialized");

    let body = serde_json::to_string(&AccountState::build_query(Variables{ id: 1 }))?;
    let response = alice_client
      .client
      .post("/graphql/")
      .header(rocket::http::Header::new("Authentication", token))
      .body(body)
      .dispatch()
      .await;

    assert_eq!(response.status(), Status::Unauthorized);
  }
}
