use super::*;

#[get("/")]
pub async fn show(current: CurrentPerson, site: &State<Site>) -> JsonResult<AccountState> {
  let asite = site.inner().clone();
  let account_state = AccountState::find_for(asite, *current.person.id()).await?;
  Ok(Json(account_state))
}

constata_lib::describe_one! {
  apitest!{ get_account_state (_db, c, alice_client)
    alice_client.signer.stories_with_signed_docs(b"alice").await;

    let account_state_response = alice_client.get_string("/account_state").await;
    let expected_response = "{\"org_id\":1,\"token_balance\":\"0\",\"missing\":\"0\",\"price_per_token\":\"1\",\"max_monthly_gift\":\"10\",\"monthly_gift_remainder\":\"7\",\"invoices\":[],\"funded_count\":3,\"funded_tokens\":\"3\",\"parked_count\":0,\"parked_tokens\":\"0\",\"funded_documents_count\":3,\"total_document_count\":3,\"total_document_tokens\":\"3\",\"pending_tyc_url\":null,\"pending_invoice_link_url\":null,}".to_string();

    assert_eq!(account_state_response, expected_response);
  }
}
