pub use constata_lib::{
  bitcoin::Address,
  error::{Error, Result},
  models::{*, document::Document},
  serde::{self, Deserialize, Serialize},
  signed_payload::SignedPayload,
};
pub use i18n::{HtmlWithLocale, Lang};

pub use rocket::{
  self,
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

pub type JsonResult<T> = Result<Json<T>>;

pub mod static_files;
pub mod bulletins;
pub mod stories;
pub mod documents;
pub mod download_proof_links;
pub mod explorer;
pub mod pubkey_domain_endorsements;
pub mod pubkeys;
pub mod payments;
pub mod account_state;
pub mod terms_acceptance;
pub mod certos;
pub mod current_person;
pub mod template;
pub mod request;
pub mod entry;
pub mod safe;
pub mod delete_parked;
pub mod create_email_credentials_tokens;
pub mod invoices;
pub mod public_certificates;

pub use current_person::{CurrentPerson, CurrentPersonAndJson};

constata_lib::describe_one! {
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  struct ApiStory { id: i32 }

  apitest!{ can_authenticate_with_legacy_auth (_db, c, client)
    let story = client.signer.stories_with_signed_docs(b"alice").await.pop().unwrap();
    let show_path = format!("/stories/{}", &story.attrs.id);
    client.assert_unauthorized_get(&show_path).await;
    let auth = client.make_legacy_auth_header(&show_path, "2030-07-07T15:45:36.860161242Z");
    let shown_legacy: ApiStory = serde_json::from_str(
      &client.get_response_with_auth(&show_path, auth).await.into_string().await.unwrap()
    )?;
    assert_eq!(shown_legacy.id, story.attrs.id);
    let shown: ApiStory = client.get(&show_path).await;
    assert_eq!(shown.id, story.attrs.id);
  }

  apitest!{ checks_if_signed_payload_is_signed_correctly (_db, _c, alice_client)
    let response = alice_client.get_response_with_bad_auth_signature("/stories/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
}
