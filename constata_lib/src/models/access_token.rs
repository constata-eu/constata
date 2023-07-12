use super::*;

model!{
  state: Site,
  table: access_tokens,
  struct AccessToken {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(varchar)]
    token: String,
    #[sqlx_model_hints(access_token_kind)]
    kind: AccessTokenKind,
    #[sqlx_model_hints(timestamptz)]
    auto_expires_on: Option<UtcDateTime>,
    #[sqlx_model_hints(boolean, default)]
    expired: bool,
  },
  queries {
    find_active("token = $1 AND NOT expired", token: String),
    expirable("NOT expired AND auto_expires_on IS NOT NULL AND auto_expires_on < now()")
  },
  belongs_to {
    Org(org_id),
    Person(person_id),
  }
}

impl AccessTokenHub {
  pub async fn create(&self, person: &Person, kind: AccessTokenKind, duration_days: Option<i64>) -> sqlx::Result<AccessToken> {
    self.insert(InsertAccessToken{
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      token: MagicLink::make_random_token(),
      kind,
      auto_expires_on: duration_days.map(|d| Utc::now() + Duration::days(d)),
    }).save().await
  }

  pub async fn expire_all_old_access_tokens(self) -> sqlx::Result<Self> {
    for access_token in self.expirable().all().await? {
      access_token.update().expired(true).save().await?;
    }

    Ok(self)
  }
}

impl AccessToken {
  pub fn allows(&self, action: &str) -> bool {
    match self.kind() {
      AccessTokenKind::VerifyEmail => matches!(
        action,
        "EmailAddressVerification" |
        "createEmailAddressVerification"
      ),
      AccessTokenKind::InvoiceLink => matches!(
        action,
        "InvoiceLink" |
        "createInvoiceLink"
      ),
      AccessTokenKind::DownloadProofLink => matches!(
        action,
        "DownloadProofLink" |
        "Proof" |
        "updateDownloadProofLink" |
        "deleteDownloadProofLink" |
        "AbridgedProofZip"
      ),
      AccessTokenKind::VcPrompt => matches!(
        action,
        "createKioskVcRequest" |
        "KioskVcRequest"
      ),
    }
  }

  pub async fn expire(&self) -> sqlx::Result<Self> {
    self.clone().update().expired(true).save().await
  }
}

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[sqlx(type_name = "access_token_kind", rename_all = "snake_case")]
pub enum AccessTokenKind {
  VerifyEmail,
  InvoiceLink,
  DownloadProofLink,
  VcPrompt,
}

impl sqlx::postgres::PgHasArrayType for AccessTokenKind {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_access_token_kind")
  }
}

describe! {
  dbtest!{ expire_old_access_tokens (site, c)
    let alice = c.alice().await;
    let person = alice.person().await;
    expire_old_and_assert_amount_of_expired_and_not_expired(&site, 0, 0).await?;

    let mut access_tokens = vec![];

    for kind in [
      AccessTokenKind::DownloadProofLink,
      AccessTokenKind::VerifyEmail,
      AccessTokenKind::InvoiceLink,
      AccessTokenKind::DownloadProofLink,
      AccessTokenKind::VerifyEmail,
      AccessTokenKind::InvoiceLink,
    ] {
      access_tokens.push(site.access_token().create(&person, kind, Some(30)).await?);
    }

    access_tokens[0].clone().update().auto_expires_on(Some(Utc::now() - Duration::hours(1))).save().await?;
    expire_old_and_assert_amount_of_expired_and_not_expired(&site, 1, 5).await?;

    access_tokens[1].clone().update().auto_expires_on(Some(Utc::now() - Duration::hours(1))).save().await?;
    expire_old_and_assert_amount_of_expired_and_not_expired(&site, 2, 4).await?;

    access_tokens.push(site.access_token().create(&person, AccessTokenKind::InvoiceLink, Some(30)).await?);
    expire_old_and_assert_amount_of_expired_and_not_expired(&site, 2, 5).await?;

    for access_token in access_tokens {
      access_token.update().auto_expires_on(Some(Utc::now() - Duration::hours(1))).save().await?;
    }
    expire_old_and_assert_amount_of_expired_and_not_expired(&site, 7, 0).await?;

    site.access_token().create(&person, AccessTokenKind::InvoiceLink, Some(30)).await?;
    expire_old_and_assert_amount_of_expired_and_not_expired(&site, 7, 1).await?;
  }

  pub async fn expire_old_and_assert_amount_of_expired_and_not_expired(site: &Site, expired: usize, not_expired: usize) -> ConstataResult<()> {
    site.access_token().expire_all_old_access_tokens().await?;
    assert_eq!(site.access_token().select().expired_eq(true).all().await?.len(), expired);
    assert_eq!(site.access_token().select().expired_eq(false).all().await?.len(), not_expired);
    Ok(())
  }
}
