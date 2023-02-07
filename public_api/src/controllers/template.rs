use super::*;

#[get("/<id>/download_payload")]
pub async fn download_payload(id: i32, current: CurrentPerson, site: &State<Site>) -> Result<Vec<u8>> {
  site.template()
    .select()
    .id_eq(&id)
    .person_id_eq(current.person.id())
    .deletion_id_is_set(false)
    .one().await?
    .payload().await
}


constata_lib::describe_one! {
  use std::env;

  fulltest!{ certos_public_api_gets_template_payload (_db, _c, alice_client, _chain)
    let template = set_up_template(&alice_client.signer).await?;
    let download = alice_client.get_bytes(
      &format!("/template/{}/download_payload", template.id())
    ).await;

    assert_eq!(download, template.payload().await?);
  }

  apitest!{ certos_public_api_cannot_see_someone_elses_template_payload (_db, c, alice_client)
    let template = set_up_template(&c.bob().await).await?;
    
    let path = format!("/template/{}/download_payload", template.id());
    let response = alice_client.get_response(&path).await;
    assert_eq!(response.status(), Status::NotFound);
  }
  apitest!{ cannot_access_after_org_deletion (_db, _c, alice_client)
    let template = set_up_template(&alice_client.signer).await?;
    alice_client.signer.make_org_deletion(b"person deletion").await;

    let path = format!("/template/{}/download_payload", template.id());
    let response = alice_client.get_response(&path).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
  async fn set_up_template(signer: &SignerClient) ->  Result<Template> {
    let template_file = std::fs::read(&format!("{}/../constata_lib/src/test_support/resources/certos_template.zip", env::current_dir()?.display()))?;
    Ok(signer.make_template(template_file).await)
  }
}
