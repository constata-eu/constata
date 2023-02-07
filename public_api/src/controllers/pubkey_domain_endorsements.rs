use super::*;
use super::pubkey_domain_endorsement::Flow;

#[post("/", data = "<form>")]
pub async fn create(
  form: Json<PubkeyDomainEndorsementForm>,
  site: &State<Site>,
) -> JsonResult<Flow> {
  Ok(Json(form.save(&site).await?.into_inner().flow()))
}

#[get("/")]
pub async fn index(
  current: CurrentPerson,
  site: &State<Site>,
) -> JsonResult<Vec<Flow>> {
  Ok(Json(site.pubkey_domain_endorsement()
    .select()
    .person_id_eq(current.person.id())
    .deletion_id_is_set(false)
    .all().await?
    .into_iter()
    .map(|p| p.flow() )
    .collect()
  ))
}

constata_lib::describe_one! {
  use crate::test_support::PublicApiClient;
  #[derive(PartialEq, Debug, serde::Deserialize)]
  struct ApiEndorsementFlow { id: i32 }

  apitest!{ creates_and_lists_endorsements (_site, c, alice_client)
    let created: ApiEndorsementFlow = set_up_pubkey_domain_endorsement(&alice_client).await;

    alice_client.assert_unauthorized_get("/pubkey_domain_endorsements").await;
    let list: Vec<ApiEndorsementFlow> = alice_client.get("/pubkey_domain_endorsements").await;
    assert_eq!(&list[0], &created);
  }
  apitest!{ cannot_access_after_org_deletion (_db, _c, alice_client)
    set_up_pubkey_domain_endorsement(&alice_client).await;
    alice_client.signer.make_org_deletion(b"person deletion").await;

    let response = alice_client.get_response("/pubkey_domain_endorsements").await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  async fn set_up_pubkey_domain_endorsement(alice_client: &PublicApiClient) ->  ApiEndorsementFlow {
    alice_client.post("/pubkey_domain_endorsements",
      serde_json::json![{
        "signed_payload": alice_client.signer.signed_payload(b"https://constata.eu")
      }].to_string()
    ).await
  }
}
