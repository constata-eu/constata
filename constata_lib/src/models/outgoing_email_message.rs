use super::*;
use crate::{ Site, Result };
use serde_with::serde_as;
use i18n::Context;

model!{
  state: Site,
  table: outgoing_email_messages,
  #[serde_as]
  struct OutgoingEmailMessage {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    email_address_id: i32,
    #[sqlx_model_hints(timestamptz, default)]
    sent_at: Option<UtcDateTime>,
    kind: OutgoingEmailMessageKind,
  },
  queries {
    not_sent("sent_at IS NULL"),
  },
  belongs_to {
    Person(person_id),
    EmailAddress(email_address_id),
  }
}

impl OutgoingEmailMessageHub {
  pub async fn create(
    &self,
    person: &Person,
    email_address: &EmailAddress,
    kind: OutgoingEmailMessageKind
  ) -> sqlx::Result<OutgoingEmailMessage> {
    self.insert(InsertOutgoingEmailMessage{
      kind,
      person_id: person.attrs.id,
      email_address_id: email_address.attrs.id,
    }).save().await
  }
}

type HtmlAndAttachments = (String, Vec<(String, Vec<u8>, String)>);

impl OutgoingEmailMessage {
  pub async fn mark_sent(self) -> Result<()> {
    self.update().sent_at(Some(Utc::now())).save().await?;
    Ok(())
  }

  pub async fn render_html(&self) -> Result<HtmlAndAttachments> {
    match self.kind() {
      OutgoingEmailMessageKind::Welcome => 
        self.render_welcome().await,
      OutgoingEmailMessageKind::KycRequestReceived =>
        self.render_kyc_request_received().await,
      OutgoingEmailMessageKind::EmailVerification =>
        self.render_email_verification().await,
    }
  }

  pub async fn render_welcome(&self) -> Result<HtmlAndAttachments> {
    let credentials = self.person().await?.pubkey().await?.and_then(|o| o.into_credentials());
    let mut context = Context::new();
    context.insert("has_credentials", &credentials.is_some());
    let html = self.render( "emails/in_layout/welcome.html", Some(context)).await?;

    let attach = if let Some(c) = credentials {
      vec![("constata_credentials.json".into(), serde_json::to_string(&c)?.into_bytes(), "application/json".into())]
    } else {
      vec![]
    };

    Ok((html, attach))
  }

  pub async fn render_kyc_request_received(&self) -> Result<HtmlAndAttachments> {
    Ok((self.render("emails/in_layout/kyc_request_received.html", None).await?, vec![]))
  }

  pub async fn render_email_verification(&self) -> Result<HtmlAndAttachments> {
    Ok((self.render("emails/in_layout/email_verification.html", None).await?, vec![]))
  }

  pub async fn render(&self, template: &str, extra_context: Option<Context>) -> Result<String> {
    let person = self.person().await?;
    let account_state = person.org().await?.account_state().await?;
    let address = self.email_address().await?;

    let mut context = extra_context.unwrap_or_else(|| Context::new() );
    context.insert("url_to_tyc", &account_state.pending_tyc_url);
    context.insert("url_to_verify_email", &address.link_to_verify().await);
    context.insert("keep_private", &address.attrs.keep_private);

    Ok(i18n::render(person.attrs.lang, template, &context)?)
  }
}

describe! {
  use crate::models::{site::Environment, pubkey::Credentials};

  dbtest!{ sends_welcome_email (_site, c)
    let mut renders = assert_email(&c,
      OutgoingEmailMessageKind::Welcome,
      "welcome_email",
      vec!["Le damos la bienvenida a Constata", "Adjuntamos sus credenciales"],
      vec!["Welcome to Constata", "Find attached your password-protected credentials"],
    ).await;

    assert_eq!(renders[0].1.len(), 1);
    let (name, bytes, mime) = renders.pop().unwrap().1.pop().unwrap();
    assert_eq!(name, "constata_credentials.json");
    assert_eq!(mime, "application/json");
    let creds: Credentials = serde_json::from_str(std::str::from_utf8(&bytes)?)?;
    assert_that!(&creds, structure![Credentials {
      address: rematch("mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx"),
      encrypted_key: rematch("this_is_not"),
      public_key: rematch("039b6347398505f5ec93826dc61c19f47c66c0283ee9be980e29ce325a0f4679ef"),
      environment: eq(Environment::Development),
    }]);
  }

  dbtest!{ sends_kyc_request_received_email (_site, c)
    assert_email(&c,
      OutgoingEmailMessageKind::KycRequestReceived,
      "kyc_request_received",
      vec!["Recibimos su pedido de verificación", "Necesitamos que confirme recepción de este correo"],
      vec!["We have received your request to verify", "We need to confirm that you have received"],
    ).await;
  }

  dbtest!{ sends_email_verification_email (_site, c)
    assert_email(&c,
      OutgoingEmailMessageKind::EmailVerification,
      "email_verification",
      vec!["solicitó utilizar esta casilla de correo electrónico", "Confirmar recepción"],
      vec!["you requested to use this email address", "Confirm email"],
    ).await;
  }

  async fn assert_email(
    c: &TestDb,
    kind: OutgoingEmailMessageKind,
    artifact_name: &str,
    es_texts: Vec<&str>,
    en_texts: Vec<&str>
  ) -> Vec<HtmlAndAttachments> {
    let signer = c.alice_no_money_no_tyc().await;
    let mut person = signer.person().await;
    let address = signer.make_email("alice@example.com").await;
    let mut renders = vec![];

    for (lang, texts) in &[(i18n::Lang::Es, es_texts), (i18n::Lang::En, en_texts)] {
      person = person.update().lang(*lang).save().await.unwrap();
      let message = c.site.outgoing_email_message().create(&person, &address, kind).await.unwrap();
      let render = message.render_html().await.unwrap();
      std::fs::write(&format!("../target/artifacts/{}_{}.html", artifact_name, lang.code()), &render.0).unwrap();
      for t in texts {
        assert_that!(&render.0, rematch(t));
      }
      renders.push(render);
    }

    renders
  }
}
