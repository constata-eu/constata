use super::*;
use serde_with::serde_as;

model!{
  state: Site,
  table: email_addresses,
  #[serde_as]
  struct EmailAddress {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    address: String,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[serde(with = "Base64Standard")]
    evidence: Vec<u8>,
    evidence_hash: String,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: i32,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
    #[sqlx_model_hints(int4)]
    access_token_id: Option<i32>,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz)]
    verified_at: Option<UtcDateTime>,
    #[sqlx_model_hints(boolean)]
    keep_private: bool,
  },
  belongs_to {
    Org(org_id),
    OrgDeletion(deletion_id),
    Person(person_id),
    AccessToken(access_token_id),
  }
}

impl EmailAddressHub {
  pub async fn validate_new_email(&self, address: &str, person_id: &PersonId) -> ConstataResult<bool> {
    match self.state.email_address().select().address_eq(&address.to_string()).verified_at_is_set(true).optional().await? {
      Some(email) => {
        if email.person_id() == person_id { return Ok(true); }
        Ok(false)
      },
      None => Ok(true)
    }
  }

  pub async fn create(&self, person: Person, address: &str, evidence: Vec<u8>, verified: bool, keep_private: bool) -> ConstataResult<EmailAddress> {
    if !validator::validate_email(address) {
      return Err(Error::validation("address","not_an_email"));
    }

    if !self.validate_new_email(address, person.id()).await? {
      return Err(Error::validation("uniqueness","address_already_taken"));
    }

    let evidence_hash = hasher::hexdigest(&evidence);
    let access_token = self.state.access_token()
      .create(&person, AccessTokenKind::VerifyEmail, Some(30)).await?;

    Ok(self.insert(InsertEmailAddress{
      address: address.to_string(),
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      access_token_id: Some(access_token.attrs.id),
      verified_at: if verified { Some(Utc::now()) } else { None },
      evidence_hash,
      evidence,
      keep_private,
    }).save().await?)
  }

  pub async fn create_with_new_org(self, address: &str, evidence: Vec<u8>, verified: bool, lang: i18n::Lang, keep_private: bool) -> ConstataResult<EmailAddress> {
    let tx = self.transactional().await?;

    let org = tx.state.org()
      .insert(Default::default())
      .save_and_subscribe(lang)
      .await?;

    let email = tx.create(org.admin().await?, address, evidence, verified, keep_private).await?;

    tx.commit().await?;

    Ok(email)
  }

  pub async fn verify_with_token(self, token: &AccessToken) -> ConstataResult<EmailAddress> {
    let mut found = self.select().access_token_id_eq(token.attrs.id).one().await?;

    if found.attrs.verified_at.is_none() {
      found = found.update().verified_at(Some(Utc::now())).save().await?;
      token.expire().await?;
      self.state.db.execute(sqlx::query!(
        "DELETE FROM email_addresses WHERE address = $1 AND verified_at IS NULL AND id != $2",
        found.address(),
        found.id(),
      )).await?;
    }

    Ok(found)
  }
}

impl EmailAddress {
  pub async fn full_url(&self) -> Option<String> {
    Some(format!("{}#/verify_email/{}", &self.state.settings.url, self.access_token().await.ok()??.token()))
  }

  pub fn is_verified(&self) -> bool {
    self.attrs.verified_at.is_some()
  }

  pub async fn link_to_verify(&self) -> Option<String> {
    if self.is_verified() { None } else { Some(self.full_url().await?) }
  }
}

describe! {
  use crate::Error;
  use i18n::Lang;

  dbtest!{ creates_an_email_address_from_form (site, _c)
    let email_address = site.email_address().create_with_new_org(
      "bob@example.com",
      b"an email we received by them".to_vec(),
      false,
      i18n::Lang::Es,
      false,
    ).await?;

    assert_that!(&email_address.attrs, structure!(EmailAddressAttrs {
      address: eq("bob@example.com".to_string()),
      evidence: eq(b"an email we received by them".to_vec()),
      bulletin_id: eq(1),
      person_id: eq(1),
      keep_private: eq(false),
    }));
  }

  dbtest!{ fails_to_create_if_email_is_invalid (site, _c)
    let result = site.email_address().create_with_new_org(
      "blablabla",
      b"an email we received by them".to_vec(),
      false,
      i18n::Lang::Es,
      false,
    ).await;

    assert_that!(&result.unwrap_err().to_string(), rematch("ValidationError"));
  }

  dbtest!{ fails_to_create_email_address_if_already_exists (site, _c)
    let org = site.org().insert(Default::default()).save_and_subscribe(Lang::Es).await?;
    let admin = org.admin().await?;
    let first_address = admin.create_or_update_email_address(
      &"bob@example.com".to_string(), true
    ).await?;

    // If email is not verified, I can use that email to register.
    let second_address = site.email_address().create_with_new_org(
      "bob@example.com", b"test".to_vec(), false, Lang::Es, false
    ).await?;
    let third_address = site.email_address().create_with_new_org(
      "bob@example.com", b"test".to_vec(), false, Lang::Es, false
    ).await?;

    // Once the email is verified, I can't use that email to register.
    assert_eq!("#1", &org.name_for_on_behalf_of().await?);
    let verified = site.email_address().verify_with_token(
      &first_address.access_token().await?.expect("to have an access token")
    ).await?;
    assert_eq!(verified.address(), &org.name_for_on_behalf_of().await?);
    assert_that!(first_address.access_token().await?.unwrap().expired());
    let failed_address = site.email_address().create_with_new_org(
      "bob@example.com", b"test".to_vec(), false, i18n::Lang::Es, false
    ).await;
    assert_that!( &failed_address.unwrap_err(), structure![ Error::Validation {
      field: rematch("uniqueness"),
      message: rematch("address_already_taken")
    }]);


    // Once an email is verified, if another user had register with that email is going to be deleted.
    assert_that!(second_address.reloaded().await.is_err());
    assert_that!(third_address.reloaded().await.is_err());

    // If I have a verified address and I register a new unverified address,
    // is going to keep using for emails the verified address until the new one is verified.
    // It will show the new address in the Constata web app.
    let new_address = admin.create_or_update_email_address(
      &"bob_second_address@example.com".to_string(), true
    ).await?;
    assert_eq!("bob@example.com", &org.name_for_on_behalf_of().await?);
    assert_eq!("bob_second_address@example.com", admin.last_email_address().await?.address());
    site.email_address().verify_with_token(
      &new_address.access_token().await?.expect("to have an access token")
    ).await?;
    assert_eq!("bob_second_address@example.com", &org.name_for_on_behalf_of().await?);

    // If I have an old verified email is going to lock for others the creation of the same one.
    let failed_address = site.email_address().create_with_new_org(
      "bob@example.com", b"test".to_vec(), false, Lang::Es, false
    ).await;
    assert_that!( &failed_address.unwrap_err(), structure![ Error::Validation {
      field: rematch("uniqueness"),
      message: rematch("address_already_taken")
    }]);

    // If I have an old verified email, I can change it back but I going to need to verify it again.
    let same_first_address = admin.create_or_update_email_address(
      &"bob@example.com".to_string(), true
    ).await?;
    assert_eq!("bob_second_address@example.com", &org.name_for_on_behalf_of().await?);
    site.email_address().verify_with_token(
      &same_first_address.access_token().await?.expect("to have an access token")
    ).await?;
    assert_eq!("bob@example.com", &org.name_for_on_behalf_of().await?);
    assert_eq!(*same_first_address.keep_private(), true);

    // If I try to register my currently verified email address with a different keep private, it just updates the keep private field.
    let updated_address = admin.create_or_update_email_address(
      &"bob@example.com".to_string(), false
    ).await?;
    assert_eq!(*same_first_address.reloaded().await?.keep_private(), false);
    assert_eq!(updated_address, same_first_address.reloaded().await?);

  }
}
