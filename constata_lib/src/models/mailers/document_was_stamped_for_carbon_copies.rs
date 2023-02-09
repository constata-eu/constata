use crate::{Site, Result};
use serde::{Serialize};
use chrono::Utc;

#[derive(Debug, Serialize)]
pub struct DocumentWasStampedForCarbonCopies {
  pub email_subject: String,
  pub timestamp_date: String,
  pub download_link: String,
  pub copyright_year: String,
  pub on_behalf_of: String,
  pub custom_message: Option<String>,
  pub person_logo_url: Option<String>,
  pub lang: i18n::Lang,
}

impl DocumentWasStampedForCarbonCopies {
  pub async fn new(site: &Site, document_id: &str, custom_message: Option<String>) -> Result<Self> {
    let doc = site.document().find(&document_id.to_string()).await?;
    let lang = doc.story().await?.attrs.lang;
    let accepted = doc.clone().in_accepted()?;
    let published = accepted.bulletin().await?.in_published()?;
    let download_link = doc.get_or_create_download_proof_link(30).await?.safe_env_url().await?;
    let org = doc.org().await?;
    let on_behalf_of = org.name_for_on_behalf_of().await?;

    Ok(DocumentWasStampedForCarbonCopies{
      download_link,
      on_behalf_of,
      lang,
      email_subject: accepted.into_inner().base_document_part().await?.attrs.friendly_name,
      copyright_year: Utc::now().format("%Y").to_string(),
      custom_message,
      timestamp_date: published.block_time().format("%F %R").to_string(),
      person_logo_url: org.attrs.logo_url,
    })
  }

  pub fn render_html(&self) -> Result<String> {
    Ok(i18n::render_from_serialize(self.lang, "emails/document_was_stamped_for_carbon_copies.html", &self)?)
  }
}

describe! {
  regtest!{ renders_cc_emails (site, c, mut chain)
    let alice = c.alice().await.add_funds().await;
    alice.org().await
      .update()
      .public_name(Some("DAO Education".to_string()))
      .logo_url(Some("https://constata-public.ams3.digitaloceanspaces.com/logo-daoe.png".to_string()))
      .save().await?;

    let doc = alice.signed_document(&vec![1; 1024 * 1024 * 10]).await;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let mail = DocumentWasStampedForCarbonCopies::new(&site, &doc.id(), Some("\
      <b>Because</b> IPO built many reinvested all-time-high for lots of block height,
      Ethereum threw away many dead cat bounce behind a anarcho-capitalism,
      or although Litecoin returns some burned crypto of lots of private chain,
      Tezos stuck few minimum faucet at few block! Golem thought lots of arbitrage
      for many validator, therefore, Tezos was few zero knowledge proof after
      few fundamental analysis. OmiseGo thought few digital signature, however,
      because Cardano should be the hot flippening during a do your own research,
      Mt. Gox is lots of robust difficulty! Although Zcash counted some trusted
      Lambo at lots of initial coin offering, Decred thinking!".to_string())
    ).await?;
    let content_message = mail.render_html()?;
    std::fs::write("../target/artifacts/content_cc_message.html", &content_message).unwrap();
    assert_that!(&content_message, rematch("IPO built many"));
    assert_that!(&content_message, rematch("de parte de <b>DAO Education</b>"));
    assert_that!(&content_message, rematch("&lt;b&gt;Because&lt;&#x2F;"));


    let mail = DocumentWasStampedForCarbonCopies::new(&site, &doc.id(), None).await?;
    let content = mail.render_html()?;
    std::fs::write("../target/artifacts/content_cc.html", &content).unwrap();
    assert_that!(&content, rematch("La empresa Constata.EU le transmite este mensaje"));
  }

  regtest!{ renders_in_english (site, c, mut chain)
    let alice = c.alice().await.add_funds().await;
    alice.org().await
      .update()
      .public_name(Some("DAO Education".to_string()))
      .logo_url(Some("https://constata-public.ams3.digitaloceanspaces.com/logo-daoe.png".to_string()))
      .save().await?;

    let doc = alice.signed_document(&vec![1; 1024 * 1024 * 10]).await;
    doc.story().await?.update().lang(i18n::Lang::En).save().await?;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let mail = DocumentWasStampedForCarbonCopies::new(&site, &doc.id(), Some("hello".to_string())).await?;
    let content = mail.render_html()?;
    std::fs::write("../target/artifacts/content_cc_message_english.html", &content).unwrap();
    assert_that!(&content, rematch("Constata, a trust service provider, transmits"));
  }
}
