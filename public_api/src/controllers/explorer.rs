use super::*;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use constata_lib::bitcoin::Txid;
use std::str::FromStr;
#[get("/<id>")]
pub async fn show(
  site: &State<Site>,
  id: String,
) -> ConstataResult<Json<bitcoincore_rpc::json::GetRawTransactionResult>> {
  let txid = Txid::from_str(&id).map_err(|_| Error::validation("txid", "invalid txid"))?;
  let client = Client::new(
    site.settings.bitcoin_rpc_uri.clone(),
    Auth::UserPass(site.settings.bitcoin_rpc_user.clone(), site.settings.bitcoin_rpc_pass.clone()),
  )?;
  Ok(Json(client.get_raw_transaction_info(&txid, None)?))
}

constata_lib::describe_one! {
  fulltest!{ gets_transaction_info_from_blockchain (site, c, client, mut chain)
    client.signer.signed_documents(b"alice").await;

    client.assert_get_error(
      "/explorer/733f3905dd51be1baa55539f8f7d19672998357268e78633f38231bb2143fdaf",
      Status::NotFound,
      "Not found"
    ).await;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let published = site.bulletin().find(&1).await?.in_published()?;
    let tx: String  = client
      .client
      .get(&format!("/explorer/{}", published.raw_transaction_hash()))
      .dispatch()
      .await
      .into_string()
      .await
      .expect("explorer result");
    assert_that!(&tx, rematch(&published.hash()));
  }
}
