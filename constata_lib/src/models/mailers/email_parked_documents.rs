use super::super::{Invoice, ParkedReminder};
use crate::Result;
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct EmailParkedDocuments {
  pub missing_tokens: f64,
  pub has_enough_tokens: bool,
  pub total_price: f64,
  pub buy_tokens_link: Option<String>,
  pub url_to_tyc: Option<String>,
  pub token_balance: f64,
  pub parked_count: i64,
  pub funded_documents_count: i64,
  pub total_document_count: i64,
  pub invoices: Vec<Invoice>,
  pub parked_documents_urls: Vec<(String, String)>,
  pub lang: i18n::Lang,
}

impl EmailParkedDocuments {
  pub async fn new(parked_reminder: &ParkedReminder) -> Result<Self> {
    use num_traits::ToPrimitive;

    let org = parked_reminder.org().await?;
    let person = org.admin().await?;
    let lang = person.attrs.lang;
    let account_state = org.account_state().await?;
    let subscription = org.subscription_or_err().await?;
    let missing_tokens = account_state.missing.ceil().to_f64().expect("no loss");
    let token_balance = account_state.token_balance.ceil().to_f64().expect("no loss");

    Ok(EmailParkedDocuments{
      buy_tokens_link: account_state.pending_invoice_link_url,
      missing_tokens,
      lang,
      has_enough_tokens: missing_tokens == 0.0,
      token_balance,
      parked_count: account_state.parked_count,
      funded_documents_count: account_state.funded_documents_count,
      total_document_count: account_state.total_document_count,
      invoices: account_state.invoices,
      total_price: (subscription.price_per_token() * account_state.missing).to_f64().expect("no loss"),
      url_to_tyc: account_state.pending_tyc_url,
      parked_documents_urls: account_state.parked_documents_urls,
    })
  }

  pub fn render_html(&self) -> Result<String> {
    Ok(i18n::render_from_serialize(self.lang, "emails/in_layout/email_parked_documents.html", &self)?)
  }
}

describe! {
  use chrono::{Utc, TimeZone, Duration};
  dbtest!{ sends_email_parked_documents (site, c)
    let signers = vec![c.bob().await, c.robert().await, c.enterprise().await, c.alice_no_money_no_tyc().await];
    let mut counter = 0;
    let mut docs = vec![];
    for signer in signers.iter() {
      let email = format!("example_{counter}@gmail.com");
      signer.make_email(&email).await;
      docs.push(signer.signed_document(&vec![1; 1024 * 1024 * 10]).await);
      counter += 1;
    }
    let date = Utc.with_ymd_and_hms(2022, 8, 1, 13, 0, 0).unwrap();
    signers[3].stories_with_signed_docs(&vec![1; 1024 * 1024 * 10]).await;
    docs[2].clone().update().created_at(date - Duration::days(3)).save().await?;
    docs[3].clone().update().created_at(date - Duration::days(3)).save().await?;

    site.parked_reminder().create_new_reminders(date).await?;
    let persons_ready_to_be_reminded_about_parked = site.parked_reminder().not_sent().await?;
    assert_eq!(persons_ready_to_be_reminded_about_parked.len(), 2);

    counter = 0;
    for person in &persons_ready_to_be_reminded_about_parked {
      let mail = EmailParkedDocuments::new(&person).await?;
      let content = mail.render_html()?;
      let path = format!("../target/artifacts/email_parked_documents_{}.html", counter);
      std::fs::write(&path, &content)?;
      assert_that!(&content, rematch("¡Hola! Te recordamos que tienes"));
      counter += 1;
    }

    let reminder = &persons_ready_to_be_reminded_about_parked[0];
    reminder.org().await?.admin().await?.update().lang(i18n::Lang::En).save().await?;
    let english_mail = EmailParkedDocuments::new(&reminder).await?;
    let content = english_mail.render_html()?;
    assert_that!(&content, rematch("Hello! This is a reminder"));
  }
}
