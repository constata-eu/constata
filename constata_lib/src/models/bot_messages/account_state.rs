use super::super::{Invoice, Person, Subscription};
use crate::Result;
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct AccountStateMessage {
  pub give_welcome: bool,
  pub token_balance: f64,
  pub missing_tokens: f64,
  pub parked_count: i64,
  pub buy_tokens_link: Option<String>,
  pub subscription: Subscription,
  pub url_to_tyc: Option<String>,
  pub funded_documents_count: i64,
  pub total_document_count: i64,
  pub invoices: Vec<Invoice>,
  pub parked_documents_urls: Vec<(String, String)>,
  pub lang: i18n::Lang,
}

impl AccountStateMessage {
  pub async fn new(person: &Person, give_welcome: bool) -> Result<Self> {
    use num_traits::ToPrimitive;

    let org = person.org().await?;

    let account_state = org.account_state().await?;
    let subscription = org.subscription_or_err().await?;
    let missing_tokens = account_state.missing.ceil().to_f64().expect("no loss");
    let token_balance = account_state.token_balance.ceil().to_f64().expect("no loss");

    Ok(AccountStateMessage{
      give_welcome,
      buy_tokens_link: account_state.pending_invoice_link_url,
      missing_tokens,
      token_balance,
      subscription,
      parked_count: account_state.parked_count,
      funded_documents_count: account_state.funded_documents_count,
      total_document_count: account_state.total_document_count,
      invoices: account_state.invoices,
      url_to_tyc: account_state.pending_tyc_url,
      parked_documents_urls: account_state.parked_documents_urls,
      lang: person.attrs.lang,
    })
  }

  pub fn render_instant_message(&self) -> Result<String> {
    Ok(i18n::render_from_serialize(self.lang, "instant_messages/account_state.txt", &self)?)
  }
}
