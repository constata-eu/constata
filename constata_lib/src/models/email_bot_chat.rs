use super::*;
use crate::{
  Site, Error, Result,
  models::mailers::*,
};

model!{
  state: Site,
  table: email_bot_chats,
  struct EmailBotChat {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    story_id: i32,
    #[sqlx_model_hints(varchar)]
    first_document_id: String,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(varchar)]
    thread_id: String,
    #[sqlx_model_hints(varchar)]
    message_id: String,
    #[sqlx_model_hints(text)]
    subject: String,
    #[sqlx_model_hints(timestamptz, default)]
    greeting_sent_at: Option<UtcDateTime>,
  },
  queries {
    ready_to_send("greeting_sent_at IS NULL AND (
      SELECT b.state = 'published' FROM documents d
        LEFT JOIN bulletins b ON d.bulletin_id = b.id
        WHERE d.id = email_bot_chats.first_document_id
    )")
  },
  has_many {
    EmailBotChatParticipant(email_bot_chat_id),
  },
  belongs_to {
    Story(story_id),
    Person(person_id),
    Document(first_document_id),
  }
}

model!{
  state: Site,
  table: email_bot_chat_participants,
  struct EmailBotChatParticipant {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    email_bot_chat_id: i32,
    #[sqlx_model_hints(text)]
    address: String,
  },
  belongs_to {
    EmailBotChat(email_bot_chat_id)
  }
}

impl EmailBotChat {
  pub async fn mark_sent(self) -> Result<EmailBotChat> {
    if self.greeting_sent_at().is_some() {
      return Ok(self)
    }

    let sendable = match self.document().await?.in_accepted() {
      Err(_) => false,
      Ok(accepted) => accepted.bulletin().await
        .map(|b| b.is_published())
        .unwrap_or(false),
    };

    if !sendable {
      return Err(Error::validation("sent_at", "cannot_mark_as_sent"));
    }

    Ok(self.update().greeting_sent_at(Some(Utc::now())).save().await?)
  }

  pub async fn render_mailer_html(&self) -> Result<String> {
    EmailBotChatGreeting::new(&self.state, self.first_document_id()).await?.render_html()
  }
}

impl InsertEmailBotChatParticipantHub {
  pub async fn validate_and_save(self) -> Result<EmailBotChatParticipant> {
    if !validator::validate_email(self.address()) {
      return Err(Error::validation("address","not_an_email"));
    }

    Ok(self.save().await?)
  }
}


describe! {
  regtest!{ creates_an_email_bot_chat_and_marks_greeting_as_sent (ref site, c, mut chain)
    let all = ||{ async { site.email_bot_chat().select().all().await.unwrap() }};
    let ready_to_send = ||{ async { site.email_bot_chat().ready_to_send().all().await.unwrap() }};

    let chat = insert(&c).await.save().await.unwrap();
    chat.state.email_bot_chat_participant().insert(InsertEmailBotChatParticipant{
      email_bot_chat_id: chat.attrs.id,
      address: "yo@nubis.im".to_string(),
    }).save().await?;

    let found = vec![chat.clone()];

    // Cannot mark as sent before the bulletin is done.
    assert_that!(
      &chat.clone().mark_sent().await.unwrap_err(),
      structure![ Error::Validation{ message: rematch("cannot_mark_as_sent") } ]
    );

    assert_that!(&all().await, eq(found.clone()));
    assert!(&ready_to_send().await.is_empty());

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    assert_that!(&all().await, eq(found.clone()));
    assert_that!(&ready_to_send().await, eq(found.clone()));
    assert!(chat.clone().mark_sent().await.is_ok());

    assert!(&site.email_bot_chat().select().greeting_sent_at_is_set(false).all().await?.is_empty());
    assert!(&ready_to_send().await.is_empty());

    // Mark as sent again does not change the sent date.
    let old_greeting_sent_at = chat.reloaded().await?.attrs.greeting_sent_at.clone().unwrap();
    assert_eq!(
      chat.reloaded().await?.mark_sent().await?.attrs.greeting_sent_at.unwrap(),
      old_greeting_sent_at
    );
  }

  dbtest!{ cannot_create_duplicates (_site, c)
    let it = insert(&c).await;

    assert!(it.clone().save().await.is_ok());
    assert_that!(
      &it.clone().save().await.unwrap_err().to_string(),
      rematch("duplicate key value violates unique constraint \"email_bot_chats_thread_id_key\"")
    );
  }

  dbtest!{ cannot_create_for_nonexistent_document (site, _c)
    let err = site.email_bot_chat().insert(InsertEmailBotChat{
      story_id: 1,
      first_document_id: "11".to_string(),
      person_id: 1,
      thread_id: "8238923983298".to_string(),
      message_id: "8238923983298".to_string(),
      subject: "Some subject".to_string(),
    }).save().await.unwrap_err();

    assert_that!(
      &err.to_string(),
      rematch("violates foreign key constraint \"fk_story\"")
    );
  }

  async fn insert(c: &TestDb) -> InsertEmailBotChatHub {
    let document = c.alice().await.add_funds().await.signed_document(b"Hello").await;
    c.site.email_bot_chat().insert(InsertEmailBotChat{
      story_id: document.attrs.story_id,
      first_document_id: document.attrs.id.clone(),
      person_id: document.attrs.person_id,
      thread_id: "8238923983298".to_string(),
      message_id: "8238923983298".to_string(),
      subject: "Some subject".to_string(),
    })
  }
}
