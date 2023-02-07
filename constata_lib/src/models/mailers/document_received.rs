use super::super::{Document, Invoice};
use crate::{Site, Result};
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct DocumentReceived {
  pub subject: String,
  pub document_friendly_name: String,
  pub give_welcome: bool,
  pub accepted: bool,
  pub cost: f64,
  pub price_per_token: f64,
  pub eta: Option<i64>,
  pub gift: Option<f64>,
  pub missing_tokens_for_other: f64,
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

impl DocumentReceived {
  pub async fn new(site: &Site, document: Document) -> Result<Self> {
    use num_traits::ToPrimitive;

    let person = document.person().await?;
    let lang = person.attrs.lang;
    let org = document.org().await?;
    let account_state = org.account_state().await?;
    let subscription = org.subscription_or_err().await?;

    let give_welcome = account_state.total_document_count == 1;
    let cost = document.cost().clone();
    let eta = document.eta().await;
    let gift = match document.gift_id() {
      Some(id) => Some(site.gift().find(id).await?.attrs.tokens),
      _ => None
    };
    let subject = if give_welcome {
      i18n::t!(lang, mailer_welcome_message_subject)
    } else {
      i18n::t!(lang, mailer_document_received_subject)
    }.to_string();

    let missing_tokens = account_state.missing.ceil().to_f64().expect("no loss");
    let token_balance = account_state.token_balance.ceil().to_f64().expect("no loss");

    Ok(DocumentReceived{
      give_welcome,
      eta,
      subject,
      lang,
      buy_tokens_link: account_state.pending_invoice_link_url,
      missing_tokens,
      has_enough_tokens: missing_tokens == 0.0,
      token_balance,
      parked_count: account_state.parked_count,
      funded_documents_count: account_state.funded_documents_count,
      total_document_count: account_state.total_document_count,
      invoices: account_state.invoices,
      cost: cost.ceil().to_f64().expect("no loss"),
      gift: gift.map(|g| g.ceil().to_f64().expect("no loss")),
      accepted: document.is_accepted(),
      missing_tokens_for_other: (account_state.missing - cost).ceil().to_f64().expect("no loss"),
      price_per_token: subscription.price_per_token().to_f64().expect("no loss"),
      total_price: (subscription.price_per_token() * account_state.missing).to_f64().expect("no loss"),
      document_friendly_name: document.friendly_name().await?,
      url_to_tyc: account_state.pending_tyc_url,
      parked_documents_urls: account_state.parked_documents_urls,
    })
  }

  pub fn render_html(&self) -> Result<String> {
    Ok(i18n::render_from_serialize(self.lang, "emails/in_layout/document_received.html", &self)?)
  }

  pub fn render_instant_message(&self, group_chat_name: Option<&str>) -> Result<String> {
    let mut context = i18n::Context::from_serialize(&self)?;
    context.insert("group_chat_name", &group_chat_name);
    Ok(i18n::render(self.lang, "instant_messages/document_received.txt", &context)?)
  }
}

describe! {
  use super::super::super::{PaymentSource, Decimal};
  dbtest!{ sends_welcome_message_the_first_time (raw_site, c)
    let robert = c.robert().await;
    let robert_ref = &robert;
    let site = &raw_site;

    let make_mail = |bytes: Vec<u8>| async move {
      let doc = robert_ref.signed_document(&bytes).await;
      let mail = DocumentReceived::new(&site, doc).await?;
      let content = mail.render_html()?;
      Result::Ok((mail, content))
    };

    let (first, first_content) = make_mail(b"Hello World!".to_vec()).await?;
    assert!(first.give_welcome);
    assert_that!(&first_content, rematch("Recuerda que cuando nos escribes o pones en copia"));
    assert_that!(&first_content, rematch("debes aceptar nuestros <b>Términos y Condiciones</b>."));
    assert_that!(&first_content, rematch("tu mensaje estará certificado en unos minutos"));
    assert_that!(&first_content, rematch("El costo de certificar este mensaje es"));
    assert_that!(&first_content, rematch("<b>Te bonificamos 1 token</b>"));
    std::fs::write("../target/artifacts/first_document_received.html", &first_content)?;

    robert_ref.clone().accept_terms_and_conditions().await;

    let (second, second_content) = make_mail(vec![1; 1024 * 1024 * 9]).await?;
    assert!(!second.give_welcome);
    assert_that!(&second_content, rematch("El tiempo estimado de certificación es"));
    assert_that!(&second_content, rematch("<b>Te bonificamos 9 tokens</b>"));
    assert_that!(&second_content, not(rematch("Recuerda que cuando nos escribes o pones en copia")));
    assert_that!(&second_content, not(rematch("debes aceptar nuestros <b>Términos y Condiciones</b>.")));
    assert_that!(&second_content, not(rematch("Comprar Tokens")));
    std::fs::write("../target/artifacts/second_document_received.html", &second_content)?;

    //Third email, does not gift any more tokens, requests payment.
    let (_, third_content) = make_mail(vec![1; 1024 * 1024 * 4]).await?;
    assert_that!(&third_content, not(rematch("El tiempo estimado de certificación es")));
    assert_that!(&third_content, not(rematch("<b>Te bonificamos 9 tokens</b>")));
    assert_that!(&third_content, rematch("Comprar Tokens"));
    assert_that!(&third_content, not(rematch("para certificar todos los documentos que tienes pendientes.")));
    assert_that!(&third_content, rematch("Descartar "));
    std::fs::write("../target/artifacts/third_document_received.html", &third_content)?;

    let mut invoice = site.invoice().once(&robert_ref.org().await, PaymentSource::BankBbva, Decimal::new(10,0)).await?;

    //Fourth email requests payment again, and mentions previous documents.
    let (_, fourth_content) = make_mail(vec![1; 1024 * 1024 * 5]).await?;
    assert_that!(&fourth_content, not(rematch("El tiempo estimado de certificación es")));
    assert_that!(&fourth_content, not(rematch("Te bonificamos")));
    assert_that!(&fourth_content, rematch("para certificar todos los documentos que tienes pendientes."));
    assert_that!(&fourth_content, rematch("Descartar "));
    assert_that!(&fourth_content, rematch("Pagos pendientes"));
    std::fs::write("../target/artifacts/fourth_document_received.html", &fourth_content)?;

    invoice.make_payment(None).await?;
    robert.clone().add_funds().await;

    //Fifth email has no special text. No payment needed. No welcome. No gift.
    let (_, fifth_content) = make_mail(vec![1; 1024 * 1024 * 3]).await?;
    assert_that!(&fifth_content, rematch("El tiempo estimado de certificación es"));
    assert_that!(&fifth_content, not(rematch("Te bonificamos")));
    assert_that!(&fifth_content, not(rematch("Comprar Tokens")));
    std::fs::write("../target/artifacts/fifth_document_received.html", &fifth_content)?;
  }

  dbtest!{ request_payment_after_gifting_many_tokens_on_welcome (site, c)
    let robert = c.robert().await;

    let doc = robert.signed_document(&vec![7; 1024 * 1024 * 20]).await;
    let mail = DocumentReceived::new(&site, doc).await?;
    let content = mail.render_html()?;
    assert_that!(&content, rematch("Recuerda que cuando nos escribes o pones en copia"));
    assert_that!(&content, rematch("debes aceptar nuestros <b>Términos y Condiciones</b>."));
    assert_that!(&content, rematch("para completar tu certificación deberás adquirir los tokens necesarios."));
    assert_that!(&content, rematch("<b>Te bonificamos 10 tokens</b>"));
    assert_that!(&content, rematch("Comprar Tokens"));
    std::fs::write("../target/artifacts/sixth_document_received.html", &content)?;
  }

  dbtest!{ sends_in_english (site, c)
    c.alice().await;
    let bot = c.bot().await.accept_terms_and_conditions().await;
    bot.person().await.update().lang(i18n::Lang::En).save().await?;
    let doc = bot.witnessed_email_with_story().await;

    let mail = DocumentReceived::new(&site, doc).await?;
    let content = mail.render_html()?;

    std::fs::write("../target/artifacts/document_received_english.html", &content)?;
    assert_that!(&content, rematch("Message sent by CONSTATA EU"));
  }
}
