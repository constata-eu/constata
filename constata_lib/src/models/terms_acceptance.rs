use super::*;
use crate::{Site, Result};

model!{
  state: Site,
  table: terms_acceptances,
  struct TermsAcceptance {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    token: String,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(timestamptz)]
    created_at: Option<UtcDateTime>,
    #[sqlx_model_hints(timestamptz, default)]
    accepted: Option<UtcDateTime>,
    #[sqlx_model_hints(bytea, default)]
    evidence: Option<Vec<u8>>,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: Option<i32>,
    #[sqlx_model_hints(varchar, default)]
    hash: Option<String>,
  },
  belongs_to {
    Person(person_id),
    Org(org_id),
  },
}

impl TermsAcceptance {
  pub fn full_url(&self) -> String {
    format!("{}/terms_acceptance/{}", &self.state.settings.url, self.token())
  }

  pub fn is_needed(&self) -> bool {
    self.attrs.accepted.is_none()
  }

  pub async fn accept(&self, evidence: &[u8]) -> Result<TermsAcceptance> {
    let accepted = if self.is_needed() {
      let (tx, draft) = self.state.bulletin().current_draft().await?;
      let updated = self.clone().update()
        .accepted(Some(Utc::now()))
        .evidence(Some(evidence.to_vec()))
        .bulletin_id(Some(*draft.id()))
        .hash(Some(hasher::hexdigest(evidence)))
        .save()
        .await?;
      tx.commit().await?;
      updated
    } else {
      self.clone()
    };

    self.state.org()
      .find(accepted.org_id())
      .await?
      .account_state()
      .await?
      .fund_all_documents()
      .await?;

    Ok(accepted)
  }
}

impl InsertTermsAcceptance {
  pub fn from_person(person: &Person) -> Self {
    Self{
      person_id: *person.id(),
      org_id: *person.org_id(),
      token: MagicLink::make_random_token(),
      created_at: Some(Utc::now())
    }
  }
}

describe! {
  dbtest!{ verify_and_accept_terms (_site, c)
    let person = c.make_person().await;
    let tyc = person.get_or_create_terms_acceptance().await?;
    assert_eq!(tyc.person_id(), person.id());
    assert!(tyc.is_needed());
    assert!(tyc.bulletin_id().is_none());
    assert!(tyc.hash().is_none());
    assert!(tyc.evidence().is_none());

    let accepted = tyc.accept(b"some http request here").await?;
    assert!(!accepted.is_needed());
    assert_eq!(accepted.bulletin_id(), &Some(1));
    assert_eq!(accepted.hash(), &Some("af611e894bf3cfb9f8784eb94b7a3319e9793379a02c5032be56d01f56fb84c3".to_string()));
    assert_eq!(accepted.evidence(), &Some(b"some http request here".to_vec()));
  }
}
