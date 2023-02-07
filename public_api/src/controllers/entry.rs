use super::*;

#[get("/<id>/download_payload")]
pub async fn download_payload(id: i32, current: CurrentPerson, site: &State<Site>) -> Result<Vec<u8>> {
  site.entry()
    .select()
    .id_eq(&id)
    .person_id_eq(current.person.id())
    .deletion_id_is_set(false)
    .one().await?
    .storage_fetch().await
}


constata_lib::describe_one! {

  fulltest!{ certos_public_api_gets_entry_payload (_db, _c, alice_client, _chain)
    let entry = alice_client.signer.make_entry_and_sign_it().await;
    let download = alice_client.get_bytes(
      &format!("/entry/{}/download_payload", entry.id())
    ).await;

    assert_eq!(download, entry.payload().await?);
  }

  apitest!{ certos_public_api_cannot_see_someone_elses_entry_payload (_db, c, alice_client)
    let entry = c.bob().await.make_entry_and_sign_it().await;
    
    let path = format!("/entry/{}/download_payload", entry.id());
    let response = alice_client.get_response(&path).await;
    assert_eq!(response.status(), Status::NotFound);
  }

  apitest!{ cannot_access_after_org_deletion (_db, _c, alice_client)
    let entry = alice_client.signer.make_entry_and_sign_it().await;
    alice_client.signer.make_org_deletion(b"person deletion").await;

    let path = format!("/entry/{}/download_payload", entry.id());
    let response = alice_client.get_response(&path).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }
}
