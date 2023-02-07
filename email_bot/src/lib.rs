use google_gmail1::{
  Gmail,
  api::{
    Message,
    Scope, 
    ListMessagesResponse,
    ModifyMessageRequest,
    UserMessageListCall,
    Label,
  }
};
use std::default::Default;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use i18n::*;

use constata_lib::{
  models::{
    Site,
    Document,
    DocumentSource,
    email_callback::EmailCallback,
    email_bot_chat::*,
    mailers::*,
    Story,
    PersonId,
    ParkedReminder,
    OutgoingEmailMessage,
    OutgoingEmailMessageKind,
  }
};
use log::*;
use mailparse::{MailAddr, SingleInfo};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  GmailError(#[from] google_gmail1::Error),
  #[error(transparent)]
  ConstataError(#[from] constata_lib::Error),
  #[error(transparent)]
  MailParseError(#[from] mailparse::MailParseError),
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
  labels: [String; 3],
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

    let all = hub.users()
      .labels_list("me")
      .add_scope(Scope::Gmai)
      .doit().await?.1
      .labels.unwrap_or_else(|| vec![]);

    let mut ids = vec![];

    for value in &["ACE_BUSY", "ACE_DONE", "ACE_FAIL"] {
      let label: Label = match all.iter().find(|i| i.name.as_ref().unwrap() == value ) {
        Some(x) => x.clone(),
        None => {
          hub.users()
            .labels_create(Label{ name: Some(value.to_string()), ..Default::default() }, "me")
            .add_scope(Scope::Gmai)
            .doit().await?.1
        },
      };

      ids.push(label.id.unwrap());
    }

    let labels = ids.try_into().unwrap();

    Ok(Self{ hub, labels, site })
  }

  pub async fn witness_emails(&self, max_results: u32) -> MailerResult<()> {
    let mut prev_listed: Option<ListMessagesResponse> = None;
    let mut listed = self.list_messages(max_results).doit().await?.1;

    let mut messages: Vec<Message> = vec![];

    loop {
      match listed.next_page_token {
        None => {
          if let Some(Some(mut earlier)) = prev_listed.map(|x| x.messages) {
            messages.append(&mut earlier);
          }
          if let Some(mut oldest) = listed.messages {
            messages.append(&mut oldest);
          }
          break;
        },
        Some(ref token) => {
          let new_token = token.clone();
          prev_listed = Some(listed);
          listed = self.list_messages(max_results).page_token(&new_token).doit().await?.1;
        },
      }
    }

    let mut received: Vec<ReceivedMessage> = vec![];
    for m in messages.into_iter() {
      received.push(ReceivedMessage::new(m,self).await?);
    }

    received.sort_by(|a,b| a.full_message.internal_date.cmp(&b.full_message.internal_date) );

    for message in received {
      message.process().await?;
    }

    Ok(())
  }

  fn list_messages(&self, max_results: u32) -> UserMessageListCall<'_> {
    self.hub.users()
      .messages_list("me")
      .max_results(max_results)
      .add_label_ids("INBOX")
      .add_scope(Scope::Gmai)
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

    for chat in self.site.email_bot_chat().ready_to_send().all().await? {
      self.notify_chat(chat).await?;
    }

    for parked_reminder in self.site.parked_reminder().not_sent().await? {
      self.notify_parked_documents(&parked_reminder).await?;
    }

    for outgoing in self.site.outgoing_email_message().not_sent().all().await? {
      self.send_outgoing_email_message(&outgoing).await?;
    }

    Ok(())
  }

  async fn notify_chat(&self, chat: EmailBotChat) -> MailerResult<()> {
    self.send_email(EmailContent {
      recipients: chat.email_bot_chat_participant_vec().await?.into_iter().map(|c| c.attrs.address ).collect(),
      subject: format!("{}", chat.attrs.subject),
      html: chat.render_mailer_html().await?,
      attachments: vec![],
    }, Some(chat.attrs.thread_id.clone()), Some(chat.attrs.message_id.clone()), false).await?;

    chat.mark_sent().await?;

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

struct ReceivedMessage<'a> {
  email_bot: &'a EmailBot,
  id: String,
  full_message: Message,
  raw_message: Message,
  lang: Lang,
}

impl<'a> ReceivedMessage<'a> {
  async fn new(inner: Message, email_bot: &'a EmailBot) -> MailerResult<ReceivedMessage<'_>> {
    let id = inner.id.ok_or_else(|| Error::Wtf("A message must have an ID") )?;

    // Raw message has raw payload, no other message has it.
    let raw_message = email_bot.hub.users()
      .messages_get("me", &id)
      .format("raw")
      .add_scope(Scope::Gmai)
      .doit().await?.1;

    // Full message has headers, no other message has them.
    let full_message = email_bot.hub.users()
      .messages_get("me", &id)
      .add_scope(Scope::Gmai)
      .doit().await?.1;

    let subject = Self::fetch_header(&full_message, "Subject").expect("Subject is API mandatory");
    let lang = i18n::Lang::detect_from_text(&subject);

    Ok(Self{ email_bot, id, full_message, raw_message, lang })
  }

  async fn process(&self) -> MailerResult<()> {
    let [busy, done, fail] = &self.email_bot.labels;
    self.change_label("INBOX", &busy).await?;

    let next_label = match self.process_helper().await {
      Ok(_) => done,
      Err(e) => {
        error!("Error processing email {} {:?}", &e, &e);
        fail
      }
    };

    self.change_label(&busy, &next_label).await?;
    Ok(())
  }

  async fn process_helper(&self) -> MailerResult<()> {
    let author_id = self.get_or_create_author_id().await?;

    let thread_id = self.hub().users()
      .messages_get("me", &self.id)
      .add_scope(Scope::Gmai)
      .doit().await?.1
      .thread_id.expect("Thread id is API mandatory");

    if let Some(chat) = self.site().email_bot_chat().select().thread_id_eq(&thread_id).optional().await? {
      let story = chat.story().await?;
      self.make_document(&story, author_id, true).await?;
    } else {
      let story = self.site().story().create_for_email_thread(
        author_id,
        &thread_id,
        self.lang,
      ).await?;
      let doc = self.make_document(&story, author_id, false).await?;

      let chat = self.site().email_bot_chat().insert(InsertEmailBotChat{
        story_id: story.attrs.id,
        first_document_id: doc.attrs.id.clone(),
        person_id: author_id,
        thread_id: thread_id.to_owned(),
        message_id: self.header("Message-ID").unwrap_or_else(|| format!("{thread_id}@msg-id-placeholder-by-constata")),
        subject: self.header("Subject").expect("Subject is API mandatory")
      }).save().await?;

      for address in self.participants()?.into_iter() {
        self.site().email_bot_chat_participant()
          .insert(InsertEmailBotChatParticipant{ email_bot_chat_id: chat.attrs.id, address })
          .save().await?;
      }

      self.send_receipt(doc).await?;
    };

    Ok(())
  }

  async fn make_document(&self, story: &Story, author_id: PersonId, always_gift: bool) -> MailerResult<Document> {
    let payload = base64::decode_config(
      self.raw_message.raw.as_ref().ok_or_else(|| Error::Wtf("No raw payload on raw message") )?,
      base64::URL_SAFE
    )?;

    let tx = self.site().witnessed_document().transactional().await?;
    let tx_story = tx.select().state.story().find(story.id()).await?;

    let doc = tx.select().state.witnessed_document().create_from_payload(
      &tx_story,
      &payload,
      &self.evidence()?,
      author_id,
      Some(("message/rfc822".to_string(), "email".to_string())),
      DocumentSource::Email,
      always_gift,
    ).await?;

    tx.commit().await?;

    Ok(doc)
  }

  async fn get_or_create_author_id(&self) -> MailerResult<PersonId> {
    let sender = self.header("From")
      .ok_or_else(|| Error::Wtf("got email with no sender"))?;
    let sender_address = mailparse::addrparse(&sender)?
      .extract_single_info()
      .ok_or_else(|| Error::Wtf("got email with bad sender address"))?;

    match self.site().email_address().select().address_eq(&sender_address.addr).one().await {
      Ok(email_address) => Ok(email_address.attrs.person_id),
      Err(sqlx::Error::RowNotFound) => {
        Ok(self.site().email_address()
          .create_with_new_org(&sender_address.addr, self.evidence()?, true, self.lang, false).await?
          .attrs.person_id)
      },
      Err(err) => Err(err.into())
    }
  }

  fn participants(&self) -> MailerResult<Vec<String>> {
    let mut all = vec![];

    let mut add_one = |a: SingleInfo| {
      if a.addr.starts_with("ace") && a.addr.ends_with("@constata.eu") {
        return;
      }

      if all.iter().find(|x: &&SingleInfo| a.addr == x.addr).is_some() {
        return;
      }

      all.push(a);
    };

    add_one(self.from()?);

    for header in &["To", "Cc"] {
      if let Some(value) = self.header(header) {
        for addr in mailparse::addrparse(&value)?.into_inner() {
          match addr {
            MailAddr::Single(info) => add_one(info),
            MailAddr::Group(info) => {
              for a in info.addrs {
                add_one(a)
              }
            },
          }
        }
      }
    }

    Ok(all.into_iter().map(|a|{
      if let Some(name) = &a.display_name {
        format!("{} <{}>", name.replace('"', r#"\""#), a.addr)
      } else {
        a.addr.to_string()
      }
    }).collect())
  }

  fn fetch_header(m: &Message, header_name: &str) -> Option<String> {
    m.payload.as_ref()?.headers.as_ref()?
      .iter().find(|h| h.name.as_deref() == Some(header_name))
      .and_then(|x| x.value.as_ref().map(|x| x.to_string()))
  }

  fn header(&self, header_name: &str) -> Option<String> {
    Self::fetch_header(&self.full_message, header_name)
  }

  fn bot(&self) -> &EmailBot { &self.email_bot }
  fn hub(&self) -> &Gmail { &self.bot().hub }
  fn site(&self) -> &Site { &self.bot().site }

  fn from(&self) -> MailerResult<SingleInfo> {
    let header = self.header("From").ok_or_else(|| Error::Wtf("From header is mandatory") )?;
    let info = mailparse::addrparse(&header)?
     .extract_single_info()
     .ok_or_else(|| Error::Wtf("From must be singleinfo"))?;
    Ok(info)
  }

  fn evidence(&self) -> MailerResult<Vec<u8>> {
    Ok(serde_json::to_vec( &serde_json::json![{"gmail_message_id": &self.id }])?)
  }

  async fn change_label(&self, remove: &str, add: &str) -> MailerResult<()> {
    self.email_bot.hub.users().messages_modify(
      ModifyMessageRequest{
        add_label_ids: Some(vec![add.to_string()]),
        remove_label_ids: Some(vec![remove.to_string()]),
      },
      "me",
      &self.id
    ).doit().await?;
    Ok(())
  }

  async fn send_receipt(&self, doc: Document) -> MailerResult<()> {
    let maybe_email_address = self.email_bot.site.email_address()
      .select()
      .person_id_eq(doc.person_id())
      .optional().await?;

    if let Some(email_address) = maybe_email_address {
      let email = DocumentReceived::new(&self.email_bot.site, doc).await?;
      let html = email.render_html()?;

      let content = EmailContent {
        recipients: vec![email_address.attrs.address],
        subject: email.subject,
        html,
        attachments: vec![],
      };

      self.email_bot.send_email(content, None, None, true).await?;
    }

    Ok(())
  }
}

constata_lib::describe_one!{
  use google_gmail1::api::BatchDeleteMessagesRequest;
  use constata_lib::models::{UtcDateTime};
  use chrono::{Utc, TimeZone, Duration};
  
  dbtest!{ builds_a_gmail_hub (site, _c)
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;
    assert!(bot.hub.users().messages_list("me").add_label_ids("INBOX").doit().await.is_ok());
  }

  regtest!{ witnesses_emails_and_removes_from_inbox_send_email_receipt(site, c, mut chain)
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;
    let [_busy, done, _fail] = &bot.labels;
    reset_state(&bot).await;

    // There's nothing witnessed yet.
    assert_eq!(site.document().select().count().await?, 0);
    assert_eq!(site.story().select().count().await?, 0);
    assert_unsent_notifications(&site, 0).await;
    assert_email_with_label_count(&bot.hub, "SENT", 0).await;

    bot.witness_emails(12).await.unwrap();

    assert_eq!(17, site.email_bot_chat_participant().select().count().await?);

    assert_email_with_label_count(&bot.hub, "SENT", 9).await;

    assert_eq!(site.document().select().count().await?, 13);
    assert_eq!(site.story().select().count().await?, 9);
    assert_email_with_label_count(&bot.hub, "INBOX", 0).await;
    assert_email_with_label_count(&bot.hub, &done, 13).await;
    assert_unsent_notifications(&site, 9).await;

    // Trying to notify chats has no effect,
    // No bulletins have been published so there are no chat greetings to send.
    bot.handle_notify_emails().await.unwrap();
    assert_unsent_notifications(&site, 9).await;

    // Now that bulletins have been published, chat greetings should be sent.
    chain.fund_signer_wallet();
    c.accept_terms_to_all_clients().await;
    c.add_funds_to_all_clients().await;
    chain.simulate_stamping().await;
    bot.handle_notify_emails().await.unwrap();
    assert_unsent_notifications(&site, 0).await;
  }

  regtest!{ sends_email_callbacks(site, c, mut chain)
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;
    reset_state(&bot).await;

    let alice = c.alice().await;
    let address = alice.make_email("yo+alice@example.com").await.attrs.address;
    let doc = c.alice().await.accepted_document(b"hello").await;
    site.email_callback().insert(constata_lib::models::email_callback::InsertEmailCallback{
      address,
      cc: true,
      document_id: doc.as_inner().id().clone(),
      custom_message: None,
      sent_at: None,
    }).validate_and_save().await?;

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    bot.handle_notify_emails().await.unwrap();
  }

  dbtest!{ witnesses_emails_and_delete_parked (site, _c)
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;
    let [_busy, _done, _fail] = &bot.labels;
    reset_state(&bot).await;
    bot.witness_emails(3).await.unwrap();
    assert_eq!(10, site.email_bot_chat_participant().select().count().await?);

    let documents = site.document().select().all().await?;
    assert_eq!(documents.len(), 4);

    for doc in documents {
      doc.in_parked()?.delete_parked().await?;
      assert_that!(doc.reloaded().await.is_err());
    }
    
    assert_that!(site.document().select().all().await?.is_empty());
    assert_that!(site.email_bot_chat_participant().select().all().await?.is_empty());
  }

  dbtest!{ send_email_for_parked (site, _c)
    /* Para que se envie un reminder se tienen que cumplir 3 condiciones:
      -Tener un documento en parked creado hace más de 2 días.
      -Que el reminder más reciente que se envíe sea de hace:
        -Más de 4 días y sea Lunes.
        -Mas de 7 días si es cualquier otro día de la semana.
      -Que a esa persona no se le haya enviado un reminder hace más de 30 días.
    */
    let bot = EmailBot::new_with_configs(site.clone(), "src/test_support/").await?;
    let [_busy, _done, _fail] = &bot.labels;
    reset_state(&bot).await;
    bot.witness_emails(6).await.unwrap();
    assert_eq!(12, site.email_bot_chat_participant().select().count().await?);

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

  async fn assert_unsent_notifications(site: &Site, count: i64) {
    assert_eq!(site.email_bot_chat().select().greeting_sent_at_is_set(false).count().await.unwrap(), count,
      "Email bot chats {:?}", site.email_bot_chat().select().all().await.unwrap());
  }

  async fn reset_state(bot: &EmailBot) {
    let hub = &bot.hub;
    let [busy, done, fail] = &bot.labels;

    // delete SENT emails
    let sent_ids:Option<Vec<String>> = hub.users()
      .messages_list("me")
      .add_label_ids("SENT")
      .add_scope(Scope::Gmai)
      .doit().await.unwrap()
      .1.messages
      .map(|msgs| msgs.into_iter().map(|m| m.id.unwrap() ).collect() );

    let _ = hub.users().messages_batch_delete(BatchDeleteMessagesRequest { ids: sent_ids }, "me").doit().await;

    // Reset all labels, move test messages back to INBOX
    let maybe_ids: Option<Vec<String>> = hub.users()
      .messages_list("me")
      .add_scope(Scope::Gmai)
      .doit().await.expect("Message list query to work")
      .1.messages
      .map(|msgs| msgs.into_iter().map(|m| m.id.unwrap() ).collect() );

    /*
     * Uncomment this block when you need to take a look at the emails being processed to find out their ID
    for i in &maybe_ids.clone().unwrap() {
      let full_message = hub.users()
        .messages_get("me", i)
        .add_scope(Scope::Gmai)
        .doit().await.unwrap().1;
      println!("{}: {}", &full_message.id.as_ref().unwrap(), full_message.snippet.as_ref().unwrap());
    }
     */

    if maybe_ids.is_some() {
      let batch_remove_request = google_gmail1::api::BatchModifyMessagesRequest{
        add_label_ids: None,
        ids: maybe_ids,
        remove_label_ids: Some(vec!["INBOX".to_string(), busy.clone(), done.clone(), fail.clone()])
      };
      hub.users().messages_batch_modify(batch_remove_request, "me").add_scope(Scope::Gmai).doit().await.unwrap();
    }

    let batch_add_request = google_gmail1::api::BatchModifyMessagesRequest{
      add_label_ids: Some(vec!["INBOX".to_string()]),
      ids: Some(vec![
        "17f6dfe4e4f93d02",
        "17fb1249a0d2739a",
        "17f6e02dc3a55c8f",
        "17fb152e9b39a042",
        "181714c944e34864", // Thread 2, mail 3, yo@nubis
        "181714c3ca037099", // Thread 2, mail 2, gnubis@
        "181714754ff6ac43", // Thread 2, mail 1, yo@nubis
        "1817146ac450f19d", // Thread 1, mail 3, yo@nubis
        "18171455284a8e9f", // Thread 1, mail 2, gnubis@
        "1817143a59fcfb8a", // Thread 1, mail 1, yo@nubis
        "181f6facc6ea8eb6", // Gigante
        "181f715d1aa97efd", // Adjunto original que luego se va a repetir.
        "181f7165a80ea58b", // Adjunto repetido del anterior.
      ].into_iter().map(|x| x.to_string()).collect()),
      remove_label_ids: None,
    };

    hub.users().messages_batch_modify(batch_add_request, "me")
      .add_scope(Scope::Gmai)
      .doit()
      .await
      .expect("Batch modification to happen");
  }

  async fn assert_email_with_label_count(hub: &Gmail, label_id: &str, count: usize) {
    assert_eq!(
      count,
      hub.users()
        .messages_list("me")
        .add_label_ids(label_id)
        .add_scope(Scope::Gmai)
        .doit().await.unwrap()
        .1.messages
        .map( |msgs| msgs.len() )
        .unwrap_or(0),
      "Testing for messages in {}", label_id
    );
  }
}
