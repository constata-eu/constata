use super::*;
use crate::{ Site, Result, };

model!{
  state: Site,
  table: telegram_bot_updates,
  struct TelegramBotUpdate {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    update_id: i32,
    #[sqlx_model_hints(timestamptz, default)]
    synced_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    processed_at: Option<UtcDateTime>,
    #[sqlx_model_hints(varchar)]
    payload: String,
  },
  queries {
    pending_process("processed_at IS NULL")
  },
}

derive_storable!(TelegramBotUpdate, "tg-update", update_id);

impl TelegramBotUpdate {
  pub async fn put_attachment(&self, payload: &[u8]) -> Result<()> {
    self.storage_put(payload).await
  }
  pub async fn get_attachment(&self) -> Result<Vec<u8>> {
    self.storage_fetch().await
  }
}

model!{
  state: Site,
  table: telegram_bot_outgoing_messages,
  struct TelegramBotOutgoingMessage {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(bigint)]
    chat_id: i64,
    #[sqlx_model_hints(varchar)]
    message: String,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    sent_at: Option<UtcDateTime>,
    #[sqlx_model_hints(boolean, default)]
    failed: bool,
    #[sqlx_model_hints(text, default)]
    error_log: Option<String>,
  },
  queries {
    pending_send("sent_at IS NULL ORDER BY id")
  },
}

impl TelegramBotOutgoingMessageHub {
  pub async fn send(&self, chat_id: i64, message: String) -> sqlx::Result<TelegramBotOutgoingMessage> {
    self.insert(InsertTelegramBotOutgoingMessage{ chat_id, message }).save().await
  }
}

model!{
  state: Site,
  table: telegram_users,
  struct TelegramUser {
    #[sqlx_model_hints(varchar)]
    id: String,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(varchar)]
    username: Option<String>,
    #[sqlx_model_hints(varchar)]
    first_name: String,
    #[sqlx_model_hints(varchar)]
    last_name: Option<String>,
  },
  belongs_to {
    Person(person_id),
  }
}

impl TelegramUserHub {
  pub async fn create_with_new_person(self,
    id: String,
    first_name: String,
    last_name: Option<String>,
    username: Option<String>,
    lang: i18n::Lang,
  ) -> Result<TelegramUser> {
    let person = self.state.org()
      .insert(Default::default())
      .save_and_subscribe(lang)
      .await?.admin().await?.attrs;
    Ok(self.insert(InsertTelegramUser{
      id,
      person_id: person.id,
      org_id: person.org_id,
      first_name,
      last_name,
      username
    }).save().await?)
  }
}

model!{
  state: Site,
  table: telegram_bot_group_chats,
  struct TelegramBotGroupChat {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    story_id: i32,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(bigint)]
    chat_id: i64,
    #[sqlx_model_hints(varchar, default)]
    first_document_id: Option<String>,
    #[sqlx_model_hints(int4, default)]
    greeting_message_id: Option<i32>,
    #[sqlx_model_hints(timestamptz, default)]
    last_greeting_reminder: Option<UtcDateTime>,
  },
  queries {
    ready_to_send_greeting("greeting_message_id IS NULL AND (
      SELECT b.state = 'published' FROM documents d
        LEFT JOIN bulletins b ON d.bulletin_id = b.id
        WHERE d.id = telegram_bot_group_chats.first_document_id
    )"),
    ready_to_send_reminder("last_greeting_reminder < (now() - interval '12 hour') AND EXISTS (
      SELECT d.id FROM documents d
        INNER JOIN bulletins b ON d.bulletin_id = b.id AND b.state = 'published' 
        WHERE
          d.story_id = telegram_bot_group_chats.story_id
          AND b.block_time > telegram_bot_group_chats.last_greeting_reminder
    )")
  },
  belongs_to {
    Story(story_id),
    Person(person_id),
    Document(first_document_id),
    TelegramBotOutgoingMessage(greeting_message_id),
  }
}

model!{
  state: Site,
  table: telegram_bot_private_chats,
  struct TelegramBotPrivateChat {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(bigint)]
    chat_id: i64,
    #[sqlx_model_hints(int4)]
    greeting_message_id: i32,
  },
  belongs_to {
    Person(person_id),
    Org(org_id),
    TelegramBotOutgoingMessage(greeting_message_id),
  }
}

model!{
  state: Site,
  table: telegram_bot_private_chat_documents,
  struct TelegramBotPrivateChatDocument {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    document_id: String,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4, default)]
    notification_message_id: Option<i32>,
  },
  queries {
    ready_to_send_notification("notification_message_id IS NULL AND (
      SELECT b.state = 'published' FROM documents d
        LEFT JOIN bulletins b ON d.bulletin_id = b.id
        WHERE d.id = telegram_bot_private_chat_documents.document_id
    )")
  },
  belongs_to {
    Person(person_id),
    Document(document_id),
    TelegramBotOutgoingMessage(notification_message_id),
  }
}
