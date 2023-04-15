use super::*;
use constata_lib::models::{
  blockchain::PrivateKey,
};

#[post("/", data = "<signed>")]
pub async fn create(signed: CurrentPersonAndJson<SignedStoryForm>, site: &State<Site>, l: i18n::MaybeLang) -> JsonResult<StoryBundle> {
  Ok(Json(StoryBundle::from_story(&signed.json.save(&site, signed.person.person.attrs.id, l).await?).await?))
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct SignedStoryForm {
  pub signed_payload: SignedPayload,
  #[serde(default)]
  pub markers: String,
  #[serde(default)]
  pub open_until: Option<UtcDateTime>,
  #[serde(default)]
  pub filename: Option<String>,
}

impl SignedStoryForm {
  pub async fn save(&self, site: &Site, person_id: PersonId, l: i18n::MaybeLang) -> Result<Story> {
    let person = site.person().find(&person_id).await?;
    let lang = l.value.unwrap_or(person.attrs.lang);
    let tx = site.story().transactional().await?;
    let story = tx.create(person.attrs.org_id, self.open_until, self.markers.clone(), lang).await?;
    let _doc = story.state.document().create_from_signed_payload(&story, &self.signed_payload, self.filename.as_deref()).await;
    tx.commit().await?;
    Ok(story)
  }
}

#[get("/")]
pub async fn index(current: CurrentPerson, site: &State<Site>) -> JsonResult<Vec<StoryBundle>> {
  let stories = site.story().select()
    .org_id_eq(current.person.org_id())
    .deletion_id_is_set(false)
    .all().await?;
  let mut bundles = vec![];

  for story in stories {
    bundles.push(StoryBundle::from_story(&story).await?);
  }

  Ok(Json(bundles))
}

#[get("/<id>")]
pub async fn show(
  id: i32,
  current: CurrentPerson,
  site: &State<Site>,
) -> JsonResult<StoryBundle> {
  let story = org_story(&site, *current.person.org_id(), id).await?;
  Ok(Json(StoryBundle::from_story(&story).await?))
}

#[get("/<id>/html_proof")]
pub async fn html_proof(
  id: i32,
  current: CurrentPerson,
  site: &State<Site>,
  key: &State<PrivateKey>,
  lang: Lang,
) -> Result<String> {
  org_story(&site, *current.person.org_id(), id).await?
    .proof(site.settings.network, &key).await?
    .render_html(lang)
}

async fn org_story(site: &Site, org_id: i32, story_id: i32) -> Result<Story> {
  Ok(
    site.story().select()
      .org_id_eq(org_id)
      .deletion_id_is_set(false)
      .id_eq(&story_id)
      .one().await?
  )
}

constata_lib::describe_one! {
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  struct ApiStory { id: i32 }

  impl PartialEq<Story> for ApiStory {
    fn eq(&self, other: &Story) -> bool {
      self.id == *other.id()
    }
  }

  fulltest!{ creates_lists_and_gets_stories (_db, c, alice_client, mut chain)
    let created: ApiStory = alice_client.post("/stories/",
      serde_json::json![{
        "markers": "foo bar baz",
        "open_until": "2030-01-01T00:00:00.000000000Z",
        "signed_payload": alice_client.signer.signed_payload(samples::multipart_email().as_bytes()),
        "filename": "",
      }].to_string()
    ).await;

    let show_path = format!("/stories/{}", &created.id);
    alice_client.assert_unauthorized_get(&show_path).await;
    let shown: ApiStory = alice_client.get(&show_path).await;
    assert_eq!(&shown, &created);

    let proof_url = format!("/stories/{}/html_proof", shown.id);

    alice_client.assert_get_error(
      &proof_url,
      Status::NotFound,
      "will be ready when Bulletin 1 is published"
    ).await;

    alice_client.assert_get_error(
      &format!("/documents/{}/html_proof", c.site.document().select().one().await?.attrs.id),
      Status::NotFound,
      "will be ready when Bulletin 1 is published"
    ).await;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let proof = alice_client.get_string(&proof_url).await;
    assert_that!(&proof, rematch("997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5"));
    assert_that!(&proof, rematch("holis 😅"));
  }

  apitest!{ cannot_see_someone_elses_stories (_db, c, alice_client)
    let alice_stories = alice_client.signer.stories_with_signed_docs(b"alice").await;
    let bob_stories = c.bob().await.stories_with_signed_docs(b"bob").await;

    let shown: Vec<ApiStory> = alice_client.get("/stories/").await;
    assert_eq!(shown, alice_stories);

    let show_path = format!("/stories/{}", &bob_stories[0].id());
    let response = alice_client.get_response(&show_path).await;
    assert_eq!(response.status(), Status::NotFound);
  }

  apitest!{ auth_fails_if_date_is_in_the_past (_db, c, alice_client)
    let response = alice_client.get_response_with_old_auth_token("/stories/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ cannot_use_endpoints_if_correctly_signed_but_person_is_not_registered (_db, c, _alice_client)
    crate::test_support::PublicApiClient::new(c.eve().await).await
      .assert_unauthorized_get("/stories/").await;
  }

  apitest!{ forbids_if_authentication_header_is_malformed (_db, _c, alice_client)
    let response = alice_client.get_response_with_malformed_payload("/stories/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ does_not_allow_requests_with_authorization_for_another_endpoint (_db, c, alice_client)
    let response = alice_client.get_response_with_wrong_auth_path("/stories/").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ creates_with_filename (_db, c, alice_client)
    let story: ApiStory = alice_client.post("/stories/",
      serde_json::json![{
        "markers": "foo bar baz",
        "open_until": "2030-01-01T00:00:00.000000000Z",
        "signed_payload": alice_client.signer.signed_payload(&read("json_for_testing.json")),
        "filename": "json_for_testing.json",
      }].to_string()
    ).await;

    let document_part = c.site.story().find(&story.id).await?
      .documents().await?[0]
      .document_part_vec().await?[0].clone();
    
    assert_eq!(&document_part.attrs.content_type, "application/json");
    assert_eq!(&document_part.attrs.friendly_name, "document.json");
  }

  apitest!{ creates_without_filename (_db, c, alice_client)
    let story: ApiStory = alice_client.post("/stories/",
      serde_json::json![{
        "markers": "foo bar baz",
        "open_until": "2030-01-01T00:00:00.000000000Z",
        "signed_payload": alice_client.signer.signed_payload(&read("json_for_testing.json")),
      }].to_string()
    ).await;

    let document_part = c.site.story().find(&story.id).await?
      .documents().await?[0]
      .document_part_vec().await?[0].clone();
    
    assert_eq!(&document_part.attrs.content_type, "text/plain");
    assert_eq!(&document_part.attrs.friendly_name, "document.txt");
  }

  apitest!{ cannot_access_after_org_deletion (_db, _c, alice_client)
    let alice_stories = alice_client.signer.stories_with_signed_docs(b"alice").await;
    alice_client.signer.make_org_deletion(b"person deletion").await;

    let show_path = format!("/stories/{}", &alice_stories[0].id());
    let response = alice_client.get_response(&show_path).await;
    assert_eq!(response.status(), Status::Unauthorized);

    let response = alice_client.get_response("/stories").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
}
