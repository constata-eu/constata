use super::*;
use crate::{
  Site, Error, Result,
  models::mailers::*,
};

model!{
  state: Site,
  table: email_callbacks,
  struct EmailCallback {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    document_id: String,
    #[sqlx_model_hints(varchar)]
    address: String,
    #[sqlx_model_hints(boolean)]
    cc: bool,
    #[sqlx_model_hints(text)]
    custom_message: Option<String>,
    #[sqlx_model_hints(timestamptz)]
    sent_at: Option<UtcDateTime>,
  },
  queries {
    ready_to_send("sent_at IS NULL AND (
      SELECT b.state = 'published' FROM documents d
        LEFT JOIN bulletins b ON d.bulletin_id = b.id
        WHERE d.id = email_callbacks.document_id
    )")
  },
  belongs_to {
    Document(document_id),
  }
}

impl InsertEmailCallbackHub {
  pub async fn validate_and_save(self) -> Result<EmailCallback> {
    if !validator::validate_email(self.address()) {
      return Err(Error::validation("address","not_an_email"));
    }

    Ok(self.save().await?)
  }
}

impl EmailCallback {
  pub async fn mark_sent(self) -> Result<EmailCallback> {
    let sendable = match self.document().await?.in_accepted() {
      Err(_) => false,
      Ok(accepted) => accepted.bulletin().await
        .map(|b| b.is_published())
        .unwrap_or(false),
    };

    if !sendable || self.sent_at().is_some() {
      return Err(Error::validation("sent_at", "cannot_mark_as_sent"));
    }

    Ok(self.update().sent_at(Some(Utc::now())).save().await?)
  }

  pub async fn render_mailer_html(&self) -> Result<String> {
    DocumentWasStampedForCarbonCopies::new(&self.state, self.document_id(), self.custom_message().to_owned()).await?.render_html()
  }
}

describe! {
  regtest!{ creates_an_email_callback_and_sets_it_sent (ref site, c, mut chain)
    let all = ||{ async { site.email_callback().select().all().await.unwrap() }};
    let ready_to_send = ||{ async { site.email_callback().ready_to_send().all().await.unwrap() }};

    let callback = insert(&c).await.save().await.unwrap();
    let found = vec![callback.clone()];

    // Cannot mark as sent before the bulletin is done.
    assert_that!(
      &callback.clone().mark_sent().await.unwrap_err(),
      structure![ Error::Validation{ message: rematch("cannot_mark_as_sent") } ]
    );

    assert_that!(&all().await, eq(found.clone()));
    assert!(&ready_to_send().await.is_empty());

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    assert_that!(&all().await, eq(found.clone()));
    assert_that!(&ready_to_send().await, eq(found.clone()));
    assert!(callback.clone().mark_sent().await.is_ok());

    assert!(&site.email_callback().select().sent_at_is_set(false).all().await?.is_empty());
    assert!(&ready_to_send().await.is_empty());

    // Cannot mark it as sent again.
    assert_that!(
      &callback.reloaded().await?.mark_sent().await.unwrap_err(),
      structure![ Error::Validation{ message: rematch("cannot_mark_as_sent") } ]
    );
  }

  dbtest!{ cannot_create_duplicates (_site, c)
    let it = insert(&c).await;

    assert!(it.clone().save().await.is_ok());
    assert_that!(
      &it.clone().save().await.unwrap_err().to_string(),
      rematch("duplicate key value violates unique constraint \"email_callbacks_document_id_key\"")
    );
  }

  dbtest!{ cannot_create_for_nonexistent_document (site, _c)
    let err = site.email_callback().insert(InsertEmailCallback{
      document_id: "11".to_string(),
      address: "yo@example.com".to_string(),
      cc: false,
      custom_message: None,
      sent_at: None,
    }).save().await.unwrap_err();

    assert_that!(
      &err.to_string(),
      rematch("violates foreign key constraint \"email_callbacks_document_id_fkey\"")
    );
  }

  async fn insert(c: &TestDb) -> InsertEmailCallbackHub {
    let document_id = c.alice().await.add_funds().await.signed_document(b"Hello").await.id().to_string();
    c.site.email_callback().insert(InsertEmailCallback{
      document_id: document_id,
      address: "yo@example.com".to_string(),
      cc: false,
      custom_message: None,
      sent_at: None,
    })
  }
}
