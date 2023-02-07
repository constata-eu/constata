use super::*;
use crate::{
  Site, Error, Result, signed_payload::SignedPayload,
};

model!{
  state: Site,
  table: create_email_credentials_tokens,
  struct CreateEmailCredentialsToken {
    #[sqlx_model_hints(varchar)]
    id: String,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(boolean, default)]
    completed: bool,
    #[sqlx_model_hints(bytea, default)]
    evidence: Option<Vec<u8>>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Person(person_id),
    Org(org_id),
  },
}

impl CreateEmailCredentialsTokenHub {
  pub async fn create(&self, person_id: i32) -> sqlx::Result<CreateEmailCredentialsToken> {
    self.state.create_email_credentials_token()
      .insert(InsertCreateEmailCredentialsToken{
        id: MagicLink::make_random_token(),
        person_id,
        org_id: self.state.person().find(person_id).await?.attrs.org_id,
    })
    .save().await
  }
}

impl CreateEmailCredentialsToken {
  pub async fn link_pubkey(mut self, signed_payload: &SignedPayload, evidence: &[u8]) -> Result<Pubkey> {
    if *self.completed() {
      return Err(Error::validation("link_already_used", "this_credentials_token_was_already_used"));
    }

    let tx = self.state.pubkey().transactional().await?;
    let pubkey = tx.create_from_credentials_token(self.clone(), signed_payload).await?;
    self.state.db = pubkey.state.db.clone();
    self.update().completed(true).evidence(Some(evidence.to_vec())).save().await?;
    tx.commit().await?;

    Ok(pubkey)
  }

  pub fn full_url(&self) -> Option<String> {
    if *self.completed() {
      None
    } else {
      Some(format!("{}/create_email_credentials_token/{}", &self.state.settings.url, self.id()))
    }
  }
}

describe! {
  regtest!{ creates_a_credentials_token_and_update_it (site, c, _chain)
    let bot = c.bot().await.accept_terms_and_conditions().await;
    let credentials_token = site.create_email_credentials_token().create(bot.person_id).await?;
    let email_address = &site.person().find(&bot.person_id).await?.email_address_vec().await?[0];
    let payload = format!("{}-{}", email_address.address(), credentials_token.id());
    let signed_payload = c.eve().await.signed_payload(payload.as_bytes());

    let pubkey = credentials_token.clone().link_pubkey(&signed_payload, b"testing").await?;

    assert_eq!(credentials_token.reloaded().await?.attrs.completed, true);
    let person = site.person().find(credentials_token.person_id()).await?;
    assert_eq!(person.attrs.id, bot.person_id);
    assert_eq!(person.attrs.id, pubkey.attrs.person_id);

    assert_that!(
      site.email_address().select().person_id_eq(person.id()).optional().await?.is_some()
    );

    assert_that!(
      site.pubkey().select().person_id_eq(person.id()).optional().await?.is_some()
    );
  }
}
