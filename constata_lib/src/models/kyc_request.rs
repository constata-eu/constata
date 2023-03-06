use super::*;
use crate::{
  models::{
    kyc_request_evidence::*,
    PersonId, UtcDateTime, Site,
  },
  Result,
  Error,
};
use duplicate::duplicate_item;

model!{
  state: Site,
  table: kyc_requests,
  struct KycRequest {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(varchar, default)]
    state: String,
    #[sqlx_model_hints(varchar)]
    name: Option<String>,
    #[sqlx_model_hints(varchar)]
    last_name: Option<String>,
    #[sqlx_model_hints(varchar)]
    id_number: Option<String>,
    #[sqlx_model_hints(varchar)]
    id_type: Option<String>,
    #[sqlx_model_hints(timestamptz)]
    birthdate: Option<UtcDateTime>,
    #[sqlx_model_hints(varchar)]
    nationality: Option<String>,
    #[sqlx_model_hints(varchar)]
    country: Option<String>,
    #[sqlx_model_hints(varchar)]
    job_title: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_name: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_country: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_registration: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_tax_id: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_linkedin_id: Option<String>,
  },
  belongs_to{
    Person(person_id),
  },
  has_many{
    KycRequestEvidence(kyc_request_id),
  }
}

impl InsertKycRequestHub {
  pub async fn validate_and_save(self) -> Result<KycRequest> {
    let person = self.state.person().find(self.attrs.person_id).await?;

    if person.kyc_request_scope().state_eq("pending".to_string()).optional().await?.is_some() {
      return Err(Error::validation("pending_kyc_request", "this_user_already_has_a_pending_kyc_request"));
    };

    Ok(self.save().await?)
  }
}

impl KycRequest {
  pub async fn evidence(&self) -> Result<Vec<Vec<u8>>> {
    let mut evidences = vec![];
    for ev in self.kyc_request_evidence_scope().order_by(KycRequestEvidenceOrderBy::Id).all().await? {
      evidences.push(ev.contents().await?);
    }
    Ok(evidences)
  }

  pub fn flow(&self) -> Flow {
    match self.state().as_ref() {
      "pending" => Flow::Pending(Pending(self.clone())),
      _ => Flow::Processed(Processed(self.clone())),
    }
  }
}

#[duplicate_item(
  in_state         is_state         state_str       state_struct;
  [ in_pending   ] [ is_pending   ] [ "pending"   ] [ Pending   ];
  [ in_processed ] [ is_processed ] [ "processed" ] [ Processed ];
)]
impl KycRequest {
  pub fn in_state(&self) -> Result<state_struct> {
    self.flow().in_state()
  }

  pub fn is_state(&self) -> bool {
    self.state().as_str() == state_str
  }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  Pending(Pending),
  Processed(Processed),
}


#[duplicate_item(flow_variant; [ Pending ]; [ Processed ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(KycRequest);

#[duplicate_item(flow_variant; [ Pending ]; [ Processed ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn person_id(&self) -> &i32 { self.0.person_id() }
  pub fn into_inner(self) -> KycRequest { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a KycRequest { &self.0 }
}


#[duplicate_item(
  in_state         is_state         variant(i)             state_struct;
  [ in_pending   ] [ is_pending   ] [ Flow::Pending(i)   ] [ Pending   ];
  [ in_processed ] [ is_processed ] [ Flow::Processed(i) ] [ Processed ];
)]
impl Flow {
  pub fn in_state(&self) -> Result<state_struct> {
    if let variant([inner]) = self {
      Ok(inner.clone())
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a KycRequest {
    match self {
      Flow::Pending(a) => a.as_inner(),
      Flow::Processed(a) => a.as_inner(),
    }
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KycRequestProcessForm {
  pub name: bool,
  pub last_name: bool,
  pub id_number: bool,
  pub id_type: bool,
  pub birthdate: bool,
  pub nationality: bool,
  pub country: bool,
  pub job_title: bool,
  pub legal_entity_name: bool,
  pub legal_entity_country: bool,
  pub legal_entity_registration: bool,
  pub legal_entity_tax_id: bool,
  pub legal_entity_linkedin_id: bool,
  pub evidence: Vec<bool>,
}

impl KycRequestProcessForm {
  pub async fn is_empty(&self) -> bool {
    let has_something = self.name || self.last_name || self.id_number ||self.id_type ||
      self.birthdate || self.nationality || self.country || self.job_title ||
      self.legal_entity_name || self.legal_entity_country || self.legal_entity_registration ||
      self.legal_entity_tax_id || self.legal_entity_linkedin_id || self.evidence.iter().any(|&i| i);

    return !has_something;
  }
}

impl Pending {
  pub async fn process_update(self, kyc_request_process_form: KycRequestProcessForm) -> Result<KycRequest> {
    if kyc_request_process_form.is_empty().await {
      return Ok(self.0.update().state("processed".to_string()).save().await?);
    }

    let mut update_kyc = self.0.state.person().find(self.person_id()).await?.get_or_create_kyc_endorsement().await?.update();

    macro_rules! update_fields_from_kyc_endorsement {
      ($($field:ident),*) => ($(
        if kyc_request_process_form.$field {
          update_kyc = update_kyc.$field(self.0.attrs.$field.clone());
        }
      )*)
    }

    update_fields_from_kyc_endorsement![
      name, last_name, id_number, id_type, birthdate, nationality, country, job_title,
      legal_entity_name, legal_entity_country, legal_entity_registration, legal_entity_tax_id,
      legal_entity_linkedin_id
    ];

    let kyc_endorsement = update_kyc.save().await?;

    let evidences: Vec<Vec<u8>> = self.0.evidence().await?.into_iter()
      .zip(kyc_request_process_form.evidence.into_iter())
      .filter(|(_, keep)| *keep)
      .map(|(value,_)| value)
      .collect();

    kyc_endorsement.story().await?.save_evidence_and_model_changes(
      evidences,
      Some(kyc_endorsement)).await?;
    self.0.clone().update().state("processed".to_string()).save().await?;

    Ok(self.0.reloaded().await?)
  }
}

describe! {
  dbtest!{ create_kyc_request_and_accept_it (site, c)
    let alice = c.alice().await;
    let person_id = alice.person_id.unwrap();
    let kyc_request = alice.kyc_request_big().await;
    
    assert_eq!(kyc_request.attrs.last_name, Some("Buterin".to_string()));
    assert_eq!(&kyc_request.attrs.state, "pending");
    assert_eq!(kyc_request.evidence().await?.len(), 3);
    let kyc_request_evidence = kyc_request.kyc_request_evidence_vec().await?;
    assert_eq!(kyc_request_evidence.len(), 3);

    let kyc_request_process_form = alice.kyc_request_process_form(true, true, vec![true, true, true]).await;
    kyc_request.in_pending()?.process_update(kyc_request_process_form).await?;
    let kyc_endorsement = site.kyc_endorsement().select().person_id_eq(&person_id).one().await?;
    assert_eq!(&kyc_request.reloaded().await?.attrs.state, "processed");
    
    assert_eq!(kyc_endorsement.story().await?.documents().await?.len(), 4);
    assert_eq!(kyc_endorsement.last_name(), kyc_request.last_name());

    let kyc_request = alice.kyc_request_little().await;
    assert_eq!(kyc_request.attrs.last_name, Some("Nakamoto".to_string()));
    assert_eq!(kyc_request.evidence().await?.len(), 1);

    assert_that!(
      &site.kyc_request().insert(alice.make_insert_kyc_request(None).await).validate_and_save().await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("this_user_already_has_a_pending_kyc_request".to_string()) } }
    );

    let kyc_request_process_form = alice.kyc_request_process_form(true, true, vec![true]).await;
    kyc_request.in_pending()?.process_update(kyc_request_process_form).await?;
    let kyc_endorsement = kyc_endorsement.reloaded().await?;
    assert_eq!(kyc_endorsement.story().await?.documents().await?.len(), 6);
    assert_eq!(&kyc_request.reloaded().await?.attrs.state, "processed");
    assert_eq!(kyc_endorsement.last_name(), kyc_request.last_name());
  }

  dbtest!{ create_kyc_request_and_reject_it (site, c)
    let alice = c.alice().await;
    let person_id = alice.person_id.unwrap();
    let kyc_request = alice.kyc_request_big().await;
    
    assert_eq!(&kyc_request.attrs.state, "pending");
    let kyc_request_evidence = kyc_request.kyc_request_evidence_vec().await?;
    assert_eq!(kyc_request_evidence.len(), 3);

    let kyc_request_process_form = alice.kyc_request_process_form(false, false, vec![false, false, false]).await;

    kyc_request.in_pending()?.process_update(kyc_request_process_form).await?;
    assert_eq!(&kyc_request.reloaded().await?.attrs.state, "processed");
    assert_that!(site.person().find(&person_id).await?.kyc_endorsement().await?.is_none());
  }

  dbtest!{ create_kyc_request_and_partially_accept_it (site, c)
    let alice = c.alice().await;
    let person_id = alice.person_id.unwrap();
    let kyc_request = alice.kyc_request_big().await;
    
    assert_eq!(&kyc_request.attrs.state, "pending");
    let kyc_request_evidence = kyc_request.kyc_request_evidence_vec().await?;
    assert_eq!(kyc_request_evidence.len(), 3);

    let kyc_request_process_form = alice.kyc_request_process_form(false, true, vec![true, false, false]).await;

    kyc_request.in_pending()?.process_update(kyc_request_process_form).await?;
    let kyc_endorsement = site.kyc_endorsement().select().person_id_eq(&person_id).one().await?;
    assert_eq!(&kyc_request.reloaded().await?.attrs.state, "processed");
    
    assert_eq!(kyc_endorsement.story().await?.documents().await?.len(), 2);
    assert_that!(kyc_endorsement.name().is_none());
    assert_that!(kyc_endorsement.id_number().is_none());
    assert_that!(kyc_endorsement.id_type().is_none());
    assert_that!(kyc_endorsement.legal_entity_country().is_none());
    assert_eq!(kyc_endorsement.last_name(), kyc_request.last_name());
    assert_eq!(kyc_endorsement.birthdate(), kyc_request.birthdate());
    assert_eq!(kyc_endorsement.country(), kyc_request.country());
    assert_eq!(kyc_endorsement.nationality(), kyc_request.nationality());

  }
}
