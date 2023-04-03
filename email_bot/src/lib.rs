use google_gmail1::{ Gmail, api::Scope, };
use std::default::Default;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use constata_lib::{
  models::{
    Site,
    email_callback::EmailCallback,
    ParkedReminder,
    OutgoingEmailMessage,
    OutgoingEmailMessageKind,
  }
};
use log::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  GmailError(#[from] google_gmail1::Error),
  #[error(transparent)]
  ConstataError(#[from] constata_lib::Error),
  #[error(transparent)]
  MailComposeError(#[from] lettre::error::Error),
  #[error(transparent)]
  Utf8Error(#[from] std::string::FromUtf8Error),
  #[error(transparent)]
  Base64Decode(#[from] base64::DecodeError),
  #[error(transparent)]
  Sqlx(#[from] sqlx::Error),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error("WTF error {0}")]
  Wtf(&'static str),
  #[error(transparent)]
  ContentTypeErr(#[from] lettre::message::header::ContentTypeErr),
  #[error(transparent)]
  AddressError(#[from] lettre::address::AddressError),
}

pub type MailerResult<T> = std::result::Result<T, Error>;

pub struct EmailContent {
  recipients: Vec<String>,
  subject: String,
  html: String,
  attachments: Vec<(String, Vec<u8>, String)>,
}

pub struct EmailBot {
  hub: Gmail,
  site: Site,
}

impl EmailBot {
  pub async fn new(site: Site) -> MailerResult<Self> {
    Self::new_with_configs(site, "").await
  }

  async fn new_with_configs(site: Site, configs_path: &'static str) -> MailerResult<Self> {
    let credentials = format!("{configs_path}credentials.json");
    let token_cache = format!("{configs_path}tokencache.json");

    let secret = yup_oauth2::read_application_secret(credentials)
      .await
      .expect("Could not find or parse credentials.json");

    let auth = InstalledFlowAuthenticator::builder(
      secret,
      InstalledFlowReturnMethod::HTTPRedirect
    )
    .persist_tokens_to_disk(token_cache)
    .build()
    .await
    .expect("Should get auth");

    let scopes = &[Scope::Gmai];

    match auth.token(scopes).await {
      Ok(token) => println!("The token is {:?}", token),
      Err(e) => println!("error: {:?}", e),
    }

    let hub = Gmail::new(
      hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
      auth
    );

    Ok(Self{ hub, site })
  }

  fn build_email(&self, content: EmailContent, in_reply_to: Option<String>, reply_to_customer_care: bool) -> MailerResult<String> {
    use lettre::message::{Message, SinglePart, Attachment, MultiPart};

    let mut message_with_recipients = Message::builder();

    for recipient in content.recipients.into_iter() {
      message_with_recipients = message_with_recipients.to(recipient.parse().unwrap());
    }

    if let Some(a) = in_reply_to {
      message_with_recipients = message_with_recipients.references(a.clone()).in_reply_to(a);
    }

    if reply_to_customer_care {
      message_with_recipients = message_with_recipients.reply_to(
        self.site.settings.customer_care_email().parse()?
      );
    }

    let builder = message_with_recipients
      .from(self.site.settings.email_bot_sender().parse().unwrap())
      .subject(content.subject);

    let message = if content.attachments.len() > 0 {
      let mut mixed = MultiPart::mixed().singlepart(SinglePart::html(content.html.to_string()));
      for (filename, bytes, mime) in content.attachments {
        mixed = mixed.singlepart(Attachment::new(filename).body(bytes, mime.parse()?) );
      }
      builder.multipart(mixed)?
    } else {
      builder.singlepart(SinglePart::html(content.html.to_string()))?
    };

    Ok(String::from_utf8(message.formatted())?)
  }

  async fn send_email(&self, content: EmailContent, thread_id: Option<String>, message_id: Option<String>, reply_to_customer_care: bool) -> MailerResult<()> {
    let request = google_gmail1::api::Message{ thread_id, ..Default::default() };

    let email = self.build_email(content, message_id, reply_to_customer_care)?;
    self.hub.users()
      .messages_send(request, "me")
      .add_scope(Scope::Gmai)
      .upload_resumable(std::io::Cursor::new(email), "message/rfc822".parse().unwrap()).await?;

    Ok(())
  }

  pub async fn handle_notify_emails(&self) -> MailerResult<()> {
    for email_callback in self.site.email_callback().ready_to_send().all().await? {
      self.notify_callback(email_callback).await?;
    }

    for parked_reminder in self.site.parked_reminder().not_sent().await? {
      self.notify_parked_documents(&parked_reminder).await?;
    }

    for outgoing in self.site.outgoing_email_message().not_sent().all().await? {
      self.send_outgoing_email_message(&outgoing).await?;
    }

    Ok(())
  }

  async fn notify_callback(&self, email_callback: EmailCallback) -> MailerResult<()> {
    let html = email_callback.render_mailer_html().await?;
    let doc = email_callback.document().await?;
    let from = doc.org().await?.name_for_on_behalf_of().await?;
    let lang = doc.story().await?.attrs.lang;

    let subject = i18n::t!(lang, mailer_email_callback_subject, from);

    self.send_email(EmailContent {
      recipients: vec![email_callback.attrs.address.clone()],
      subject,
      html,
      attachments: vec![],
    }, None, None, true).await?;

    email_callback.mark_sent().await?;

    Ok(())
  }

  async fn notify_parked_documents(&self, parked_reminder: &ParkedReminder) -> MailerResult<()> {
    let admin = parked_reminder.org().await?.admin().await?;
    let lang = admin.attrs.lang;

    let subject = if admin.get_or_create_terms_acceptance().await?.is_needed() {
      i18n::t!(lang, mailer_parked_document_reminder_because_tyc_are_not_accepted_subject)
    } else {
      i18n::t!(lang, mailer_parked_document_reminder_because_payment_is_needed_subject)
    };
    
    self.send_email(EmailContent {
      recipients: vec![parked_reminder.attrs.address.clone()],
      subject,
      html: parked_reminder.render_parked_mailer_html().await?,
      attachments: vec![],
    }, None, None, true).await?;

    parked_reminder.clone().mark_sent().await?;

    Ok(())
  }

  async fn send_outgoing_email_message(&self, outgoing: &OutgoingEmailMessage) -> MailerResult<()> {
    let person = outgoing.person().await?;
    let lang = person.attrs.lang;
    let address = outgoing.email_address().await?;

    let subject = match outgoing.kind() {
      OutgoingEmailMessageKind::Welcome => i18n::t!(lang, mailer_welcome_after_website_signup_subject),
      OutgoingEmailMessageKind::KycRequestReceived => i18n::t!(lang, mailer_kyc_request_acknowledge_subject),
      OutgoingEmailMessageKind::EmailVerification => i18n::t!(lang, mailer_email_address_verification_subject),
    };

    let (html, attachments) = outgoing.render_html().await?;
    
    self.send_email(EmailContent {
      recipients: vec![address.attrs.address],
      subject,
      html,
      attachments,
    }, None, None, true).await?;

    outgoing.clone().mark_sent().await?;

    Ok(())
  }
}

constata_lib::describe_one!{
  dbtest!{ builds_a_gmail_hub (site, _c)
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;
    assert!(bot.hub.users().messages_list("me").add_label_ids("INBOX").doit().await.is_ok());
  }

  regtest!{ sends_email_callbacks(site, c, mut chain)
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;

    let alice = c.alice().await;
    let address = alice.make_email("yo+alice@example.com").await.attrs.address;
    let doc = c.alice().await.accepted_document(b"hello").await;
    site.email_callback().insert(constata_lib::models::email_callback::InsertEmailCallback{
      address,
      document_id: doc.as_inner().id().clone(),
      custom_message: None,
      sent_at: None,
    }).validate_and_save().await?;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    bot.handle_notify_emails().await.unwrap();
  }

  /*
  use constata_lib::models::{UtcDateTime};
  use chrono::{Utc, TimeZone, Duration};
  dbtest!{ send_email_for_parked (site, _c)
    /* Para que se envie un reminder se tienen que cumplir 3 condiciones:
      -Tener un documento en parked creado hace más de 2 días.
      -Que el reminder más reciente que se envíe sea de hace:
        -Más de 4 días y sea Lunes.
        -Mas de 7 días si es cualquier otro día de la semana.
      -Que a esa persona no se le haya enviado un reminder hace más de 30 días.
    */
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;

    // Instead of witnessing, just create 6 documents.
    bot.witness_emails(6).await.unwrap();

    let documents = site.document().select().all().await?;
    assert_eq!(documents.len(), 7);
    let date = Utc.with_ymd_and_hms(2022, 8, 1, 13, 0, 0).unwrap(); // Esta fecha es Lunes

    // Si no hay documentos parked de más de 2 días de antiguedad no se crea ningún reminder.
    create_parked_reminders(&site, &bot, date, 0).await?;

    // Cuando el documento parked de Alice tiene más de 2 días se crea 1 reminder.
    documents[0].clone().update().created_at(date - Duration::days(3)).save().await?;
    create_parked_reminders(&site, &bot, date, 1).await?;
    /* Luego de que se creó uno, no se crea otro por 2 motivos:
       -Tiene que pasar más de 4 días (si es lunes) desde que se creó el reminder más nuevo.
       -A esa persona no se le tiene que haber enviado un reminder en 30 días. */
    create_parked_reminders(&site, &bot, date, 1).await?;

    /* Si el documento parked de Bob tiene más de 3 días de antiguedad
     y los últimos reminders (no importa la persona, en este caso Alice) se crearon hace más de 4 días
     y es Lunes:
      -Se crea un reminder para Bob.
      -No para Alice porque se la notificó hace menos de 30 días */
    documents[1].clone().update().created_at(date - Duration::days(3)).save().await?;
    site.parked_reminder().select().one().await?
      .update().created_at(date - Duration::days(5)).save().await?; 
    create_parked_reminders(&site, &bot, date, 2).await?;

    /* Si actualizo el reminder de Bob a más de 30 días de antiguedad,
       y siendo que el reminder de Alice sigue teniendo 5 días de antiguedad
       y es Martes:
        -NO se crea reminder para Bob porque al no ser Lunes tiene que pasar más de 7 días
          desde que se creó el reminder más reciente.
        -Tampoco para Alice por eso y porque se la notificó hace menos de 30 días. */
    site.parked_reminder().select().org_id_eq(documents[1].org_id()).one().await?
      .update().created_at(date - Duration::days(31)).save().await?;
    let not_monday_day = date + Duration::days(1);
    create_parked_reminders(&site, &bot, not_monday_day, 2).await?;

    /* Si actualizo la fecha del reminder más reciente (el de Alice) para que tenga más de 7 días:
       -Me crea el reminder para Bob */
    site.parked_reminder().select().org_id_eq(documents[0].org_id()).one().await?
      .update().created_at(date - Duration::days(8)).save().await?;
    create_parked_reminders(&site, &bot, not_monday_day, 3).await?;

    // No se vuelve a crear ningún reminder porque ya se creó recién.
    for _ in 1..6 {
      create_parked_reminders(&site, &bot, date, 3).await?;
    }

    /* Si actualizo las fechas de todos los reminder a más de 30 días de antiguedad:
       -Me crea 2 reminder más */
    for parked_reminder in site.parked_reminder().select().all().await? {
      parked_reminder.update().created_at(date - Duration::days(31)).save().await?;
    }
    create_parked_reminders(&site, &bot, date, 5).await?;
  
    // No se vuelve a crear ningún reminder porque ya se crearon recién.
    for _ in 1..6 {
      create_parked_reminders(&site, &bot, date, 5).await?;
    }
  }

  async fn create_parked_reminders(site: &Site, bot: &EmailBot, date: UtcDateTime, amount: usize) -> MailerResult<()> {
    site.parked_reminder().create_new_reminders(date).await?;
    bot.handle_notify_emails().await.unwrap();
    assert_eq!(site.parked_reminder().select().all().await?.len(), amount);
    Ok(())
  }
  */
}
