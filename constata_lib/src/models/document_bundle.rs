use crate::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use super::{
  document::Document as DbDocument,
  bulletin,
  story_bundle::{Document, bulletins_from_ids },
};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct DocumentBundle {
  #[serde(flatten)]
  pub document: Document,
  pub bulletins: HashMap<i32, bulletin::Flow>,
  pub buy_tokens_link: Option<String>,
  pub url_to_tyc: Option<String>,
}

impl DocumentBundle {
  pub async fn from_document(db_document: &DbDocument) -> ConstataResult<DocumentBundle> {
    let (document, bulletin_ids, _) = Document::from_document(db_document).await?;

    let bulletins = bulletins_from_ids(&db_document.state, bulletin_ids).await?;

    let account_state = db_document.org().await?.account_state().await?;

    Ok(DocumentBundle{
      document,
      bulletins,
      buy_tokens_link: account_state.pending_invoice_link_url,
      url_to_tyc: account_state.pending_tyc_url
    })
  }
}

describe! {
  use crate::models::Utc;
  use chrono::TimeZone;
  use crate::models::{
    Decimal,
    access_token::AccessTokenKind,
  };
  
  dbtest!{ keeps_serializer_compatibility (site, c)
    let user = c.enterprise().await;
    let org = user.org().await;
    let mut access_token = site.access_token()
      .create(&user.person().await, AccessTokenKind::InvoiceLink, Some(30)).await?;
    access_token = access_token.update().token("hello+world".to_string()).save().await?;

    org.get_or_create_invoice_link().await?.update()
      .access_token_id(*access_token.id())
      .save().await?;
      
    let date = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    site.bulletin().select().one().await?.update().started_at(date).save().await?;

    let mut document = user.signed_document(&samples::multipart_email().as_bytes()).await;

    document = document.update().created_at(date).save().await?;

    let parked_bundle = serde_json::to_string(
      &DocumentBundle::from_document(&document).await?
    )?;

    let parked_expected = std::fs::read_to_string(
      "src/test_support/samples/document_bundle_parked_sample_regex.txt"
    )?;

    assert_that!(&parked_bundle, rematch(parked_expected.trim()));

    site.gift().give_and_trigger_updates(*org.id(), Decimal::new(10,0), "testing").await?;

    document = document.update().funded_at(Some(date)).save().await?;

    let accepted_bundle = serde_json::to_string(
      &DocumentBundle::from_document(&document.reloaded().await?).await?
    )?;

    println!("{}", &accepted_bundle);

    let accepted_expected = std::fs::read_to_string(
      "src/test_support/samples/document_bundle_accepted_sample_regex.txt"
    )?;

    assert_that!(&accepted_bundle, rematch(accepted_expected.trim()));
  }
}

