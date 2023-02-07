use crate::{Site, Result};
use serde::{Serialize};
use chrono::Utc;

#[derive(Debug, Serialize)]
pub struct EmailBotChatGreeting {
  pub timestamp_date: String,
  pub download_link: String,
  pub copyright_year: String,
  pub lang: i18n::Lang,
}

impl EmailBotChatGreeting {
  pub async fn new(site: &Site, document_id: &str) -> Result<Self> {
    let doc = site.document().find(&document_id.to_string()).await?;
    let lang = doc.story().await?.attrs.lang;
    let download_link = doc.create_download_proof_link(30).await?.safe_env_url().await?;
    Ok(EmailBotChatGreeting{
      download_link,
      lang,
      copyright_year: Utc::now().format("%Y").to_string(),
      timestamp_date: doc.in_accepted()?.bulletin().await?.in_published()?.block_time().format("%F %R").to_string(),
    })
  }

  pub fn render_html(&self) -> Result<String> {
    Ok(i18n::render_from_serialize(self.lang, "emails/in_layout/email_bot_chat_greeting.html", &self)?)
  }

  pub fn render_instant_message(&self, is_group_chat: bool, is_reminder: bool) -> Result<String> {
    let template = if is_group_chat {
      "instant_messages/group_chat_greeting.txt"
    } else {
      "instant_messages/private_chat_document_stamped.txt"
    };
    let mut context = i18n::Context::from_serialize(&self)?;
    context.insert("is_reminder", &is_reminder);
    Ok(i18n::render(self.lang, template, &context)?)
  }
}

describe! {
  regtest!{ sends_email_bot_chat_greeting (site, c, mut chain)
    let alice = c.alice().await.add_funds().await;
    let doc = alice.signed_document(&vec![1; 1024 * 1024 * 10]).await;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let mail = EmailBotChatGreeting::new(&site, &doc.id()).await?;
    let content = mail.render_html().unwrap();

    std::fs::write("../target/artifacts/email_bot_chat_greeting.html", &content)?;
    assert_that!(&content, rematch("Este mensaje ha sido certificado"));
  }

  regtest!{ sends_email_bot_chat_greeting_in_english (site, c, mut chain)
    let alice = c.alice().await.add_funds().await;
    let doc = alice.signed_document(&vec![1; 1024 * 1024 * 10]).await;
    doc.story().await?.update().lang(i18n::Lang::En).save().await?;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let mail = EmailBotChatGreeting::new(&site, &doc.id()).await?;
    let content = mail.render_html().unwrap();

    std::fs::write("../target/artifacts/email_bot_chat_greeting_english.html", &content)?;
    assert_that!(&content, rematch("This message has been certified"));
  }
}
