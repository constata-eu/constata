use super::*;

#[get("/<id>/download_payload")]
pub async fn download_payload(id: i32, current: CurrentPerson, site: &State<Site>) -> Result<Vec<u8>> {
  site.request()
    .select()
    .id_eq(&id)
    .person_id_eq(current.person.id())
    .deletion_id_is_set(false)
    .one().await?
    .storage_fetch().await
}


constata_lib::describe_one! {
  use std::env;

  fulltest!{ certos_public_api_gets_request_payload (_db, _c, alice_client, _chain)
    let request = set_up_template_and_request(&alice_client.signer).await?;
    let download = alice_client.get_string(
      &format!("/request/{}/download_payload", request.id())
    ).await;

    assert_eq!(download, String::from_utf8(request.payload().await?)?);
  }

  apitest!{ certos_public_api_cannot_see_someone_elses_request_payload (_db, c, alice_client)
    let request = set_up_template_and_request(&c.bob().await).await?;
    
    let path = format!("/request/{}/download_payload", request.id());
    let response = alice_client.get_response(&path).await;
    assert_eq!(response.status(), Status::NotFound);
  }

  apitest!{ certos_public_api_auth_fails_if_date_is_in_the_past (_db, c, alice_client)
    let request = set_up_template_and_request(&alice_client.signer).await?;
    let path = format!("/request/{}/download_payload", request.id());
    let response = alice_client.get_response_with_old_auth_token(&path).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ certos_public_api_forbids_if_authentication_header_is_malformed (_db, _c, alice_client)
    let request = set_up_template_and_request(&alice_client.signer).await?;
    let path = format!("/request/{}/download_payload", request.id());
    let response = alice_client.get_response_with_malformed_payload(&path).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ certos_public_api_does_not_allow_requests_with_authorization_for_another_endpoint (_db, c, alice_client)
    let request = set_up_template_and_request(&alice_client.signer).await?;
    let path = format!("/request/{}/download_payload", request.id());
    let response = alice_client.get_response_with_wrong_auth_path(&path).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
  apitest!{ cannot_access_after_org_deletion (_db, _c, alice_client)
    let request = set_up_template_and_request(&alice_client.signer).await?;
    alice_client.signer.make_org_deletion(b"person deletion").await;

    let path = format!("/request/{}/download_payload", request.id());
    let response = alice_client.get_response(&path).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
  async fn set_up_template_and_request(signer: &SignerClient) ->  Result<constata_lib::models::request::Request> {
    let template_file =  std::fs::read(&format!("{}/../constata_lib/src/test_support/resources/certos_template.zip", env::current_dir()?.display()))?;
    let template = signer.make_template(template_file).await;
    let request_file =  std::fs::read(&format!("{}/../constata_lib/src/test_support/resources/certos_request.csv", env::current_dir()?.display()))?;
    signer.make_request(*template.id(), request_file).await
  }
}
