use std::default::Default;
use constata_lib::{
  models::{
    mailers::{DocumentReceived, EmailBotChatGreeting},
    bot_messages::AccountStateMessage,
    Utc,
    telegram::*,
    Site,
    Document,
    Story,
    Person,
    DocumentSource,
  }
};
use log::*;
use i18n::*;

use teloxide_core::{
    prelude::*,
    requests::{HasPayload, ResponseResult},
    types::*,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Telegram(#[from] teloxide_core::RequestError),
  #[error(transparent)]
  Download(#[from] teloxide_core::errors::DownloadError),
  #[error(transparent)]
  ConstataError(#[from] constata_lib::Error),
  #[error(transparent)]
  Sqlx(#[from] sqlx::Error),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error("WTF error {0}")]
  Wtf(&'static str),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct TelegramBot {
  site: Site,
  bot: Bot,
}

impl TelegramBot {
  pub fn new(site: Site) -> Self {
    let bot = Bot::new(&site.settings.telegram_token);
    TelegramBot{ site, bot }
  }

  pub async fn get_updates(&self, offset: Option<i32>, timeout: Option<u32>) -> ResponseResult<Vec<Update>> {
    self.bot.get_updates()
      .with_payload_mut(|p| { p.offset = offset; p.timeout = timeout; })
      .send().await
  }

  pub async fn sync_updates(&self) -> Result<()> {
    let last_update: i32 = self.site.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COALESCE(MAX(update_id), 0)::int as "last!" FROM telegram_bot_updates"#,
    )).await?;

    let updates = self.get_updates(
      Some(last_update + 1 ),
      Some(self.site.settings.telegram_timeout),
    ).await?;

    for u in updates {
      self.sync_update(u).await?;
    }

    Ok(())
  }

  pub async fn sync_update(&self, u: Update) -> Result<TelegramBotUpdate> {
    use teloxide_core::net::Download;

    let update_id = u.id;

    if let Some(exists) = self.site.telegram_bot_update().select().update_id_eq(update_id).optional().await? {
      return Ok(exists);
    }

    let value = serde_json::to_value(&u)?;

    let maybe_file_id = match u.kind {
      UpdateKind::Message(msg) => {
        if let MessageKind::Common(common) = msg.kind {
          match common.media_kind {
            MediaKind::Photo(a) => {
              let mut sorted = a.photo.clone();
              sorted.sort_by(|a, b| a.file_size.cmp(&b.file_size));
              sorted.pop().map(|p| p.file_id )
            }
            MediaKind::Audio(a) => Some(a.audio.file_id),
            MediaKind::Voice(a) => Some(a.voice.file_id),
            MediaKind::Video(a) => Some(a.video.file_id),
            MediaKind::VideoNote(a) => Some(a.video_note.file_id),
            MediaKind::Document(a) => Some(a.document.file_id),
            _ => None,
          }
        } else {
          None
        }
      },
      _ => None
    };

    let tx = self.site.telegram_bot_update().transactional().await?;

    let update = tx.insert(InsertTelegramBotUpdate{
      update_id,
      payload: serde_json::to_string(&value)?,
    }).save().await?;

    if let Some(file_id) = maybe_file_id {
      let mut buffer = vec![];
      let f = self.bot.get_file(file_id).send().await?;
      {
        let mut cursor = std::io::Cursor::new(&mut buffer);
        self.bot.download_file(&f.file_path, &mut cursor).await?;
      }
      update.put_attachment(&buffer).await?;
    }

    tx.commit().await?;

    Ok(update)
  }

  pub async fn process_updates(&self) -> Result<()> {
    for u in self.site.telegram_bot_update().pending_process().all().await? {
      let tx = self.site.telegram_bot_update().transactional().await?;
      let model = tx.find(u.id()).await?;
      let update = serde_json::from_str::<Update>(&u.attrs.payload)?;
      
      if let Err(e) = (UpdateProcessor{ model: &model, update: &update }).process().await {
        error!("Error processing telegram update {} {:?}", u.id(), e);
        dbg!(e);
        continue;
      }

      model.update().processed_at(Some(Utc::now())).save().await?;
      tx.commit().await?;
    }
    Ok(())
  }

  pub async fn greet_group_chats(&self) -> Result<()> {
    for c in self.site.telegram_bot_group_chat().ready_to_send_greeting().all().await? {
      let tx = self.site.telegram_bot_group_chat().transactional().await?;
      let tx_chat = tx.find(c.id()).await?;
      let state = &tx_chat.state;

      if let Some(doc_id) = tx_chat.attrs.first_document_id.as_ref() {
        let text = EmailBotChatGreeting::new(state, &doc_id).await?.render_instant_message(true, false)?;
        let greeting = state.telegram_bot_outgoing_message().send(tx_chat.attrs.chat_id, text).await?;
        tx_chat.clone().update()
          .greeting_message_id(Some(greeting.attrs.id))
          .last_greeting_reminder(Some(Utc::now()))
          .save().await?;
      }

      tx.commit().await?;
    }
    Ok(())
  }

  pub async fn remind_group_chats(&self) -> Result<()> {
    for c in self.site.telegram_bot_group_chat().ready_to_send_reminder().all().await? {
      let tx = self.site.telegram_bot_group_chat().transactional().await?;
      let tx_chat = tx.find(c.id()).await?;
      let state = &tx_chat.state;

      if let Some(doc_id) = tx_chat.attrs.first_document_id.as_ref() {
        let text = EmailBotChatGreeting::new(state, &doc_id).await?.render_instant_message(true, true)?;
        state.telegram_bot_outgoing_message().send(tx_chat.attrs.chat_id, text).await?;
        tx_chat.clone().update()
          .last_greeting_reminder(Some(Utc::now()))
          .save().await?;
      }

      tx.commit().await?;
    }
    Ok(())
  }

  pub async fn notify_private_chat_documents(&self) -> Result<()> {
    for c in self.site.telegram_bot_private_chat_document().ready_to_send_notification().all().await? {
      let chat = if let Some(a) = c.person().await?.telegram_bot_private_chat_scope().optional().await? {
        a
      } else {
        continue;
      };

      let tx = self.site.telegram_bot_private_chat_document().transactional().await?;
      let tx_chat_doc = tx.find(c.id()).await?;
      let state = &tx_chat_doc.state;
      let text = EmailBotChatGreeting::new(state, &tx_chat_doc.attrs.document_id).await?.render_instant_message(false, false)?;
      let notification = state.telegram_bot_outgoing_message().send(chat.attrs.chat_id, text).await?;
      tx_chat_doc.clone().update().notification_message_id(Some(notification.attrs.id)).save().await?;
      tx.commit().await?;
    }

    Ok(())
  }

  pub async fn flush_outgoing_messages(&self) -> Result<()> {
    use teloxide_core::errors::RequestError::{Network, Io};

    for m in self.site.telegram_bot_outgoing_message().pending_send().all().await? {
      let tx = self.site.telegram_bot_outgoing_message().transactional().await?;
      let new = tx.find(m.id()).await?;

      match self.bot.send_message(ChatId(m.attrs.chat_id), m.attrs.message).send().await {
        Ok(_) => { new.update().sent_at(Some(Utc::now())).save().await?; },
        Err(Network(_) | Io(_)) => continue,
        Err(e) => {
          new.update()
            .sent_at(Some(Utc::now()))
            .failed(true)
            .error_log(Some(e.to_string()))
            .save().await?;
        },
      }
      tx.commit().await?;
    }

    Ok(())
  }
}

struct UpdateProcessor<'a> {
  model: &'a TelegramBotUpdate,
  update: &'a Update,
}

impl<'a> UpdateProcessor<'a> {
  async fn process(&self) -> Result<()> {
    match &self.update.kind {
      UpdateKind::Message(msg) | UpdateKind::EditedMessage(msg) => {
        let chat_id = msg.chat.id.0;
        let common = if let MessageKind::Common(c) = &msg.kind { c } else { return Ok(()) };
        let user = if let Some(u) = common.from.as_ref() {
          u
        } else {
          return Err(Error::Wtf("Common message without 'from'"))
        };

        let payload = match &common.media_kind {
          MediaKind::Text(t) => t.text.clone().into_bytes(),
          MediaKind::Audio(_) |
          MediaKind::Voice(_) |
          MediaKind::Video(_) |
          MediaKind::VideoNote(_) |
          MediaKind::Document(_) => self.model.get_attachment().await?,
          MediaKind::Photo(a) if !a.photo.is_empty() => self.model.get_attachment().await?,
          _ => {
            let message = i18n::t!(Self::user_lang(user), telegram_bot_cannot_certify_this_file);
            self.send_message(chat_id, message).await?;
            return Ok(())
          }
        };

        if msg.chat.is_private() {
          if let Some(c) = self.site().telegram_bot_private_chat().select().chat_id_eq(&chat_id).optional().await? {
            let author = c.person().await?;

            match &common.media_kind {
              MediaKind::Text(t) if t.text.len() < 50 => {
                let text = if t.text.to_lowercase().contains("ayuda") || t.text.to_lowercase().contains("help") {
                  AccountStateMessage::new(&author, false).await?.render_instant_message()?
                } else {
                  i18n::t!(author.attrs.lang, telegram_bot_message_too_short)
                };
                self.send_message( chat_id, text).await?;
                return Ok(());
              },
              _ => (),
            }

            self.create_private_chat_document(&author, &c, &payload).await?;
          } else {
            let author = self.create_author(&user).await?;
            self.create_private_chat(&author, chat_id).await?;
          }
        } else if msg.chat.is_group() {
          if let Some(c) = self.site().telegram_bot_group_chat().select().chat_id_eq(&chat_id).optional().await? {
            let author = if let Some(a) = self.get_author(&user).await? {
              a
            } else {
              self.create_author(&user).await?
            };
            self.create_group_chat_document(&author, &c, &payload).await?;
          }
        }
      },
      UpdateKind::MyChatMember(membership) => {
        let chat = &membership.chat;
        let chat_id = chat.id.0;

        if !(chat.is_group() && membership.new_chat_member.is_present()) {
          return Ok(())
        }

        if self.site().telegram_bot_group_chat().select().chat_id_eq(&chat_id).count().await? > 0 {
          return Ok(())
        }

        if let Some(author) = self.get_author(&membership.from).await? {
          self.create_group_chat(&author, chat_id, chat.title()).await?;
        } else {
          let text = i18n::t!(Self::user_lang(&membership.from), telegram_bot_do_not_add_me_to_groups_directly);
          self.send_message( chat_id, text).await?;
        }
      },
      _ => (),
    }

    Ok(())
  }

  async fn create_private_chat(&self, author: &Person, chat_id: i64) -> Result<()> {
    let text = AccountStateMessage::new(author, true).await?.render_instant_message()?;
    let greeting = self.send_message(chat_id, text).await?;
    self.site().telegram_bot_private_chat().insert(InsertTelegramBotPrivateChat{
      person_id: author.attrs.id,
      org_id: author.attrs.org_id,
      chat_id: chat_id,
      greeting_message_id: greeting.attrs.id
    }).save().await?;

    Ok(())
  }

  async fn create_private_chat_document(&self, author: &Person, chat: &TelegramBotPrivateChat, payload: &[u8]) -> Result<()> {
    let story = self.site().story().create(author.attrs.id, None, "".to_string(), author.attrs.lang).await?;
    let doc = self.create_document(&story, author, payload, false).await?;
    self.site().telegram_bot_private_chat_document().insert(InsertTelegramBotPrivateChatDocument{
      document_id: doc.attrs.id.clone(),
      person_id: author.attrs.id,
    }).save().await?;

    let message = DocumentReceived::new(self.site(), doc).await?.render_instant_message(None)?;

    self.send_message(chat.attrs.chat_id, message).await?;

    Ok(())
  }

  async fn create_group_chat(&self, author: &Person, chat_id: i64, title: Option<&str>) -> Result<()> {
    let story = self.site().story().create(author.attrs.id, None, title.unwrap_or("").to_string(), author.attrs.lang).await?;

    self.site().telegram_bot_group_chat().insert(InsertTelegramBotGroupChat{
      story_id: story.attrs.id,
      person_id: author.attrs.id,
      chat_id: chat_id,
    }).save().await?;

    Ok(())
  }

  async fn create_group_chat_document(&self, author: &Person, chat: &TelegramBotGroupChat, payload: &[u8]) -> Result<()> {
    let mut story = chat.story().await?;
    let is_first = chat.first_document_id().is_none();

    if is_first {
      story = story.update().lang(i18n::Lang::detect_from_text(&String::from_utf8_lossy(&payload))).save().await?;
    }

    let doc = self.create_document(&story, author, payload, !is_first).await?;

    if is_first {
      chat.clone().update().first_document_id(Some(doc.attrs.id.clone())).save().await?;
      let message = DocumentReceived::new(self.site(), doc).await?;
      let text = message.render_instant_message(Some(story.markers()))?;

      if let Some(private_chat) = story.org().await?.admin().await?.telegram_bot_private_chat_scope().optional().await? {
        self.send_message(private_chat.attrs.chat_id, text).await?;
      }
    }

    Ok(())
  }

  async fn send_message(&self, chat_id: i64, message: String) -> sqlx::Result<TelegramBotOutgoingMessage> {
    self.site().telegram_bot_outgoing_message().send(chat_id, message).await
  }

  async fn create_document(&self, story: &Story, author: &Person, payload: &[u8], always_gift: bool) -> Result<Document> {
    Ok(self.site().witnessed_document().create_from_payload(
      &story,
      &payload,
      b"", // ToDo: No evidence here?
      author.attrs.id,
      None,
      DocumentSource::Telegram,
      always_gift,
    ).await?)
  }

  async fn get_author(&self, user: &User) -> Result<Option<Person>> {
    match self.site().telegram_user().find_optional(&user.id.0.to_string()).await? {
      Some(u) => Ok(Some(u.person().await?)),
      _ => Ok(None)
    }
  }

  fn user_lang(user: &User) -> Lang {
    user.language_code.as_ref()
      .map(|c| if c.starts_with("es") { Lang::Es } else { Lang::En })
      .unwrap_or(Lang::Es)
  }

  async fn create_author(&self, user: &User) -> Result<Person> {
    let tg_user = self.site().telegram_user().create_with_new_person(
      user.id.0.to_string(),
      user.first_name.clone(),
      user.last_name.clone(),
      user.username.clone(),
      Self::user_lang(user),
    ).await?;

    Ok(tg_user.person().await?)
  }

  fn site(&self) -> &Site {
    &self.model.state
  }
}

constata_lib::describe_one!{
  use serde_json::*;

  regtest!{ processes_a_group_chat(site, c, mut chain)
    let bot = TelegramBot::new(site.clone());
    chain.fund_signer_wallet();

    load_session(&bot, "group_chat", 6, Some(606211739)).await;
    chain.simulate_stamping().await;

    let person = site.person().find(1).await?;

    assert_eq!(site.telegram_user().select().count().await?, 2);
    let private_chats = site.telegram_bot_private_chat().select().all().await?;
    assert_eq!(private_chats.len(), 1);
    assert_eq!(private_chats[0].attrs, TelegramBotPrivateChatAttrs{
      id: 1,
      person_id: 1,
      org_id: 1,
      chat_id: 411374551,
      greeting_message_id: 1,
    });

    assert_eq!(site.telegram_bot_group_chat().select().all().await?.len(), 1);

    let mut group_chat = site.telegram_bot_group_chat().find(1).await?;
    assert_that!(&group_chat.attrs, structure![TelegramBotGroupChatAttrs{
      id: eq(1),
      story_id: eq(1),
      person_id: eq(1),
      chat_id: eq(-727189845),
      first_document_id: maybe_some(rematch("1-29d0f97133222778e4782b25fa0eb2b6e2e86e0023474e9e0e21dae80e0a7c79-[a-f0-9]{16}")),
      greeting_message_id: eq(None),
      last_greeting_reminder: eq(None),
    }]);

    assert_sent_message(&site, 1, 411374551, "Soy el asistente virtual de Constata.eu").await;

    assert_sent_message(&site, 2, 411374551, "Voy a certificar los mensajes del chat grupal 'Manda un texto y luego una imagen'").await;

    person.get_or_create_terms_acceptance().await?.accept(b"").await?;
    chain.simulate_stamping().await;

    bot.greet_group_chats().await?;
    bot.notify_private_chat_documents().await?;

    assert_sent_message(&site, 3, -727189845, "Hola, soy el asistente").await;

    load_session(&bot, "group_chat", 8, None).await;

    chain.simulate_stamping().await;
    bot.greet_group_chats().await?;
    bot.remind_group_chats().await?;
    bot.notify_private_chat_documents().await?;

    group_chat.reload().await?;

    assert!(group_chat.last_greeting_reminder().is_some());

    assert_eq!(site.story().find(1).await?.document_vec().await?.len(), 3);

    assert!(site.telegram_bot_outgoing_message().find_optional(4).await?.is_none());

    group_chat.update().last_greeting_reminder(Some(Utc::now() - chrono::Duration::hours(15))).save().await?;
    bot.greet_group_chats().await?;
    bot.remind_group_chats().await?;

    assert_sent_message(&site, 4, -727189845, "Actualicé el certificado").await;
    c.creates_admin_user_for_development().await;
  }

  regtest!{ cannot_be_added_directly_in_group(site, c, mut chain)
    let bot = TelegramBot::new(site.clone());
    chain.fund_signer_wallet();

    load_session(&bot, "add_directly_in_group", 3, None).await;
    assert_eq!(site.person().select().count().await?, 0);
    assert_eq!(site.telegram_user().select().count().await?, 0);

    assert_sent_message(&site, 1, -749945149, "Lo siento, necesito contactar").await;
  }

  regtest!{ speaks_english(site, c, _chain)
    let bot = TelegramBot::new(site.clone());

    load_session(&bot, "english_session", 2, None).await;

    assert_sent_message(&site, 1, 411374551, "Soy el asistente virtual de Constata.eu").await;
  }

  regtest!{ processes_a_private_chat_session(site, c, mut chain)
    let bot = TelegramBot::new(site.clone());
    chain.fund_signer_wallet();

    // Session first two messages are just /start and a request for help
    load_session(&bot, "private_multifile_session", 2, Some(606211720)).await;

    // So we don't have any documents  but we have both a welcome message and account state sent.
    let person = site.person().find(1).await?;

    assert_sent_message(&site, 1, 411374551, "Soy el asistente virtual de Constata.eu").await;
    assert_sent_message(&site, 2, 411374551, "Si necesitas hablar con algún").await;

    assert!(person.document_vec().await?.is_empty());

    // Then we load the rest of these files.
    load_session(&bot, "private_multifile_session", 15, None).await;
    assert_sent_message(&site, 3, 411374551, "Voy a certificar tu mensaje").await;

    assert_eq!(person.org().await?.story_scope().count().await?, 7);

    chain.simulate_stamping().await;
  }

  async fn load_session(bot: &TelegramBot, path: &str, expected_total: i64, until: Option<i32>) {
    for line in std::fs::read_to_string(&format!("src/test_support/{}/database.jsonlines", path)).unwrap().lines() {
      let update = bot.sync_update(from_str(line).expect("Parsing update")).await.expect("Syncing update");

      if matches!(until, Some(max) if update.attrs.update_id == max) {
        break;
      }
    }
    std::process::Command::new("bash")
      .args(&["-c", &format!("cp -r src/test_support/{}/constata-local-* /tmp/", path)])
      .output()
      .unwrap();

    bot.process_updates().await.unwrap();
    assert_eq!(bot.site.telegram_bot_update().select().count().await.unwrap(), expected_total);
  }

  /* This test records a session of updates from the telegram API
   * Just invoke this test like:
   * scripts/record-telegram-bot
   */
  dbtest!{ record_session (site, _c)
    use std::io::{self, Write};
    use std::process::Command;

    if std::env::var("RECORD_TELEGRAM_SESSION").is_ok() {
      println!("Resetting Bot session");
      Command::new("rm").args(&["-r", "-f", "recorded_session"]).output().unwrap();
      let bot = TelegramBot::new(site.clone());

      if let Some(last) = bot.get_updates(None, Some(10)).await?.last() {
        bot.get_updates(Some(last.id + 1), Some(10)).await?;
      }

      loop {
        println!("Enter to process telegram updates, any key then Enter to stop and save session.");
        let mut input = String::new();
        let _ = std::io::stdout().flush();
        io::stdin().read_line(&mut input).expect("Error reading from STDIN");
        if &input != "\n" { break; }
        println!("Ok, processing.");
        bot.sync_updates().await.unwrap();
        bot.process_updates().await.unwrap();
        bot.flush_outgoing_messages().await.unwrap();
        println!("Done processing.");
      }

      let dump: String = site.telegram_bot_update()
        .select()
        .order_by(TelegramBotUpdateOrderBy::Id)
        .all().await.unwrap()
        .into_iter()
        .map(|u| format!("{}\n", u.attrs.payload) )
        .collect();

      Command::new("mkdir").args(&["-p", "recorded_session"]).output().unwrap();
      Command::new("bash")
        .args(&["-c", "cp -r /tmp/constata-local-tg-update-* recorded_session"])
        .output()
        .unwrap();
      std::fs::write("recorded_session/database.jsonlines", &dump)?;
    }
  }

  async fn assert_sent_message(site: &Site, id: i32, chat: i64, text: &str) {
    assert_that!(
      &site.telegram_bot_outgoing_message().find(&id).await.unwrap().attrs,
      structure![TelegramBotOutgoingMessageAttrs {
        chat_id: eq(chat),
        message: rematch(text)
      }]
    );
  }
}
