use super::{*, stories::SignedStoryForm};
use constata_lib::models::{
  blockchain::PrivateKey,
};

/* This endpoint is deprecated */
#[post("/", data = "<form>")]
pub async fn create(form: Json<SignedStoryForm>, site: &State<Site>, l: i18n::MaybeLang) -> JsonResult<DocumentBundle> {

  let person_id = site.pubkey().find(&form.signed_payload.signer.to_string()).await
    .map_err(|_| Error::validation("signed_payload/signer", "signer_is_unknown"))?
    .attrs
    .person_id;

  let docs = form.save(site, person_id, l).await?.documents().await?;
  Ok(Json(DocumentBundle::from_document(&docs[0]).await?))
}

#[get("/")]
pub async fn index(current: CurrentPerson, site: &State<Site>) -> JsonResult<Vec<DocumentBundle>> {
  let documents = site.document().select()
    .person_id_eq(current.person.id())
    .deletion_id_is_set(false)
    .all().await?;
  let mut bundles = vec![];

  for doc in documents {
    bundles.push(DocumentBundle::from_document(&doc).await?);
  }

  Ok(Json(bundles))
}

#[get("/<id>")]
pub async fn show(
  id: String,
  current: CurrentPerson,
  site: &State<Site>,
) -> JsonResult<DocumentBundle> {
  let doc = person_document(&site, current.person.id(), id).await?;
  Ok(Json(DocumentBundle::from_document(&doc).await?))
}

#[get("/<id>/html_proof")]
pub async fn html_proof(
  id: String,
  current: CurrentPerson,
  site: &State<Site>,
  key: &State<PrivateKey>,
  lang: Lang,
) -> Result<String> {
  person_document(&site, current.person.id(), id).await?
    .story().await?
    .proof(site.settings.network, &key).await?
    .render_html(lang)
}

#[get("/<id>/each_part_html_proof")]
pub async fn each_part_html_proof(
  id: String,
  current: CurrentPerson,
  site: &State<Site>,
  key: &State<PrivateKey>,
  lang: Lang,
) -> Result<std::fs::File> {
  person_document(&site, current.person.id(), id).await?
    .story().await?
    .proof(site.settings.network, &key).await?
    .generate_each_part_html_and_zip(lang)
}

async fn person_document(site: &Site, person_id: &PersonId, doc_id: String) -> Result<Document> {
  Ok(
    site.document().select()
      .person_id_eq(person_id)
      .deletion_id_is_set(false)
      .id_eq(&doc_id)
      .one().await?
  )
}

constata_lib::describe_one! {
  use crate::test_support::PublicApiClient;
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  pub struct ApiDocument { id: String }
  impl PartialEq<Document> for ApiDocument {
    fn eq(&self, other: &Document) -> bool {
      self.id.as_str() == other.id()
    }
  }

  fulltest!{ creates_lists_and_gets_documents (_db, c, alice_client, mut chain)
    c.alice().await.stories_with_signed_docs(b"alice").await;

    let created: ApiDocument = alice_client.post("/documents/",
      serde_json::json![{
        "signed_payload": c.alice().await.signed_payload(samples::multipart_email().as_bytes())
      }].to_string()
    ).await;

    let show_path = format!("/documents/{}", &created.id);
    alice_client.assert_unauthorized_get(&show_path).await;
    let shown: ApiDocument = alice_client.get(&show_path).await;
    assert_eq!(&shown, &created);

    let proof_url = format!("/documents/{}/html_proof", shown.id);
    let error: serde_json::Value = alice_client.get(&proof_url).await;
    assert_that!(
      &error.get("error").unwrap().as_str().unwrap().to_string(),
      rematch("will be ready when Bulletin 1 is published")
    );

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let proof = alice_client.get_string(&proof_url).await;
    assert_that!(&proof, rematch("997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5"));

    let proofs_zip = alice_client.get_bytes(
      &format!("/documents/{}/each_part_html_proof", shown.id)
    ).await;
    assert_eq!(
      zip::ZipArchive::new(std::io::Cursor::new(proofs_zip)).unwrap().by_index(0).unwrap().name(),
      "holis 😅_997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5.html"
    );
  }

  apitest!{ cannot_see_someone_elses_documents (_db, c, alice_client)
    let alice_docs = alice_client.signer.signed_documents(b"alice").await;
    let bob_docs = c.bob().await.signed_documents(b"bob").await;

    let shown: Vec<ApiDocument> = alice_client.get("/documents/").await;
    assert_eq!(shown, alice_docs);

    let show_path = format!("/documents/{}", &bob_docs[0].id());
    let response = alice_client.get_response(&show_path).await;
    assert_eq!(response.status(), Status::NotFound);
  }

  apitest!{ auth_fails_with_old_nonce (_db, c, alice_client)
    let response = alice_client.get_response_with_old_auth_token("/documents/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ cannot_use_endpoints_if_correctly_signed_but_person_is_not_registered (_db, c, _client)
    PublicApiClient::new(c.eve().await).await
      .assert_unauthorized_get("/documents/").await;
  }

  apitest!{ forbids_if_authentication_header_is_malformed (_db, _c, alice_client)
    let response = alice_client.get_response_with_malformed_payload("/documents/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ does_not_allow_requests_with_authorization_for_another_endpoint (_db, c, alice_client)
    let response = alice_client.get_response_with_wrong_auth_path("/documents/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ cannot_access_after_org_deletion (_db, _c, alice_client)
    let alice_docs = alice_client.signer.signed_documents(b"alice").await;
    alice_client.signer.make_org_deletion(b"person deletion").await;

    let show_path = format!("/documents/{}", &alice_docs[0].id());
    let response = alice_client.get_response(&show_path).await;
    assert_eq!(response.status(), Status::Unauthorized);

    let response = alice_client.get_response("/documents").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
}
