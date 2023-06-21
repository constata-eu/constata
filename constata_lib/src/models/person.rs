use super::*;
use i18n::Lang;

pub type PersonId = i32;

model!{
  state: Site,
  table: persons,
  struct Person {
    #[sqlx_model_hints(int4, default)]
    id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
    lang: Lang,
    lang_set_from: String,
    #[sqlx_model_hints(boolean)]
    admin: bool,
    #[sqlx_model_hints(boolean)]
    billing: bool,
    #[sqlx_model_hints(boolean, default)]
    suspended: bool,
  },
  belongs_to {
    Org(org_id),
  },
  has_many{
    Document(person_id),
    EmailAddress(person_id),
    Pubkey(person_id),
    PubkeyDomainEndorsement(person_id),
    KycEndorsement(person_id),
    TermsAcceptance(person_id),
    KycRequest(person_id),
    Issuance(person_id),
    Template(person_id),
    Entry(person_id),
  }
}

impl PersonHub {  
  pub async fn persons_that_are_missing_tokens_count(&self) -> ConstataResult<i64> {
    let count: i64 = self.state.db.fetch_one_scalar(sqlx::query_scalar!(r#"
      SELECT COUNT(DISTINCT d.person_id)::bigint as "count!" FROM documents d
      INNER JOIN terms_acceptances ta ON d.person_id = ta.person_id
      AND ta.accepted IS NOT NULL AND NOT d.funded
    "#)).await?;

    Ok(count)
  }

  pub async fn create(&self, org: &Org, admin: bool, billing: bool, lang: Lang, lang_set_from: &str) -> sqlx::Result<Person> {
    self.insert(InsertPerson{
      org_id: *org.id(),
      admin,
      billing,
      lang: lang,
      lang_set_from: lang_set_from.to_string() 
    }).save().await
  }
}

impl Person {
  pub async fn email_address(&self) -> sqlx::Result<Option<EmailAddress>> {
    self.email_address_scope().order_by(EmailAddressOrderBy::CreatedAt).desc(true).optional().await
  }

  pub async fn verified_email_address(&self) -> sqlx::Result<Option<EmailAddress>> {
    self.email_address_scope().verified_at_is_set(true).order_by(EmailAddressOrderBy::CreatedAt).desc(true).optional().await
  }

  pub async fn last_email_address(&self) -> sqlx::Result<EmailAddress> {
    self.email_address_scope().order_by(EmailAddressOrderBy::CreatedAt).desc(true).one().await
  }

  pub async fn pubkey(&self) -> sqlx::Result<Option<Pubkey>> {
    self.pubkey_scope().optional().await
  }

  pub async fn kyc_endorsement(&self) -> sqlx::Result<Option<KycEndorsement>> {
    self.kyc_endorsement_scope().optional().await
  }

  pub async fn get_or_create_terms_acceptance(&self) -> sqlx::Result<crate::models::terms_acceptance::TermsAcceptance> {
    let maybe_existing = self.terms_acceptance_scope().optional().await?;
  
    match maybe_existing {
      Some(existing) => Ok(existing),
      None => {
        self.state.terms_acceptance()
          .insert(InsertTermsAcceptance::from_person(self))
          .save().await
      }
    }
  }

  pub async fn get_or_create_kyc_endorsement(&self) -> sqlx::Result<KycEndorsement> {
    if let Some(kyc_endorsement) = self.kyc_endorsement().await? {
      return Ok(kyc_endorsement);
    }

    let tx = self.state.story().transactional().await?;
    let story = tx.create(self.attrs.id, None, "kyc endorsement".to_string(), Lang::En).await?;
    let kyc_endorsement = story.state.kyc_endorsement().insert(InsertKycEndorsement {
      person_id: self.attrs.id,
      org_id: self.attrs.org_id,
      story_id: story.attrs.id,
      name: None,
      last_name: None,
      id_number: None,
      id_type: None,
      birthdate: None,
      nationality: None,
      country: None,
      job_title: None,
      legal_entity_name: None,
      legal_entity_country: None,
      legal_entity_registration: None,
      legal_entity_tax_id: None,
      legal_entity_linkedin_id: None,
    }).save().await;
    tx.commit().await?;

    return kyc_endorsement;
  }

  pub async fn endorsements(&self) -> sqlx::Result<Vec<Endorsement>> {
    let mut endorsements: Vec<Endorsement> = vec![];

    if let Some(e) = self.kyc_endorsement().await? {
      endorsements.push(Endorsement::Kyc{ attrs: e.attrs });
    }

    if let Some(e) = self.verified_email_address().await? {
      endorsements.push(Endorsement::EmailAddress{ address: e.attrs.address, keep_private: e.attrs.keep_private });
    }

    endorsements.extend(self.pubkey_domain_endorsement_scope()
      .state_eq(&"accepted".to_string())
      .all().await?
      .into_iter()
      .map(|o| Endorsement::Website { url: o.attrs.domain, })
    );

    Ok(endorsements)
  }

  pub async fn endorsement_string(&self, lang: i18n::Lang, html: bool) -> ConstataResult<Option<String>> {
    if self.kyc_endorsement().await?.is_some() {
      Ok(Some(Proof::render_endorsements(self, lang, html).await?))
    } else {
      Ok(None)
    }
  }

  pub async fn can_send_email(&self) -> ConstataResult<bool> {
    Ok(!self.endorsements().await?.is_empty())
  }

  pub async fn create_or_update_email_address(&self, address: &String, keep_private: bool) -> ConstataResult<EmailAddress> {
    if let Some(mut existing) = self.email_address().await? {
      if existing.address() == address {
        if existing.attrs.keep_private != keep_private {
          existing = existing.update().keep_private(keep_private).save().await?;
        }
        return Ok(existing);
      }
    }

    self.state.email_address().create(self.clone(), address, vec![], false, keep_private).await
  }
}
