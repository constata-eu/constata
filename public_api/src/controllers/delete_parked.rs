use super::*;
use serde_json::json;


#[get("/<token>")]
pub async fn confirm(token: String, site: &State<Site>, l: Lang) -> Result<HtmlWithLocale> {
  let doc = site.document().select().delete_parked_token_eq(&token).one().await?;
  let friendly_name = doc.base_document_part().await?.attrs.friendly_name;
  let context = json!({"friendly_name": friendly_name});
  if !doc.can_be_deleted() {
    return Ok(l.html_from_serialize("public_api/documents/cannot_delete.html.tera", &context)?);
  };

  Ok(l.html_from_serialize("public_api/documents/confirm_deletion.html.tera", &context)?)
}

#[post("/<token>")]
pub async fn delete(token: String, site: &State<Site>) -> Result<()> {
  site.document().select()
    .delete_parked_token_eq(&token)
    .one().await?
    .in_parked()?
    .delete_parked().await?;

  Ok(())
}


constata_lib::describe_one! {
  test!{ can_delete_parked_document
    let c = TestDb::new().await?;
    let site = c.site.clone();
    let enterprise_client = crate::test_support::PublicApiClient::new(c.enterprise().await).await;

    let unfunded_document = enterprise_client.signer.signed_document(b"hello world").await;
    assert!(unfunded_document.is_parked());
    unfunded_document.get_or_create_delete_parked_url().await?;
    
    let url = format!(
      "/delete_parked/{}",
      unfunded_document.delete_parked_token().as_ref().unwrap()
    );

    let get_string = enterprise_client.get_string(&url).await;
    assert_that!(&get_string, rematch("Descartar Documento Detenido"));
    assert_that!(&get_string, not(rematch("Este documento no puede ser descartado.")));

    let response = enterprise_client.post_response(&url,
      serde_json::json![{
        "signed_payload": enterprise_client.signer.signed_payload(samples::multipart_email().as_bytes())
      }].to_string()
    ).await;
    assert_eq!(response.status(), Status::Ok);
    assert_that!(site.document().find(unfunded_document.id()).await.is_err());
  }


  apitest!{ cannot_delete_accepted_document (_db, _c, alice_client)
    let funded_document = alice_client.signer.signed_document(b"hello world").await;
    assert!(funded_document.is_accepted());
    funded_document.get_or_create_delete_parked_url().await?;
    
    let url = format!(
      "/delete_parked/{}",
      funded_document.delete_parked_token().as_ref().unwrap()
    );

    let get_string = alice_client.get_string(&url).await;
    assert_that!(&get_string, rematch("Este documento no puede ser descartado."));
    assert_that!(&get_string, not(rematch("Descartar Documento Detenido")));
  }
}
  
