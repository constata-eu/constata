use super::*;
use serde::{Deserialize, Serialize};
use models::{kyc_request, outgoing_email_message_kind::*};

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A full kyc request")]
#[serde(rename_all = "camelCase")]
pub struct KycRequestInput {
  pub email: String,
  pub keep_private: bool,
  pub name: String,
  pub last_name: String,
  pub id_number: Option<String>,
  pub id_type: Option<String>,
  pub birthdate: Option<UtcDateTime>,
  pub nationality: Option<String>,
  pub country: Option<String>,
  pub job_title: Option<String>,
  pub legal_entity_name: Option<String>,
  pub legal_entity_country: Option<String>,
  pub legal_entity_registration: Option<String>,
  pub legal_entity_tax_id: Option<String>,
  pub legal_entity_linkedin_id: Option<String>,
  pub evidence: Vec<KycRequestEvidenceInput>,
}

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A full kyc request")]
#[serde(rename_all = "camelCase")]
pub struct KycRequestEvidenceInput {
  pub filename: String,
  pub payload: String,
}

impl KycRequestInput {
  pub async fn process(self, context: &Context) -> FieldResult<KycRequest> {
    let person = context.site.person().find(context.person_id()).await?;

    let address = person.create_or_update_email_address(&self.email, self.keep_private).await?;

    // Process the email parts too here.
    let kyc_request = context.site.kyc_request().insert(kyc_request::InsertKycRequest {
      person_id: *person.id(),
      org_id: *person.org_id(),
      name: Some(self.name),
      last_name: Some(self.last_name),
      id_number: self.id_number,
      id_type: self.id_type,
      birthdate: self.birthdate,
      nationality: self.nationality,
      country: self.country,
      job_title: self.job_title,
      legal_entity_name: self.legal_entity_name,
      legal_entity_country: self.legal_entity_country,
      legal_entity_registration: self.legal_entity_registration,
      legal_entity_tax_id: self.legal_entity_tax_id,
      legal_entity_linkedin_id: self.legal_entity_linkedin_id,
    }).validate_and_save().await?;

    for e in &self.evidence {
      kyc_request.state.kyc_request_evidence().create(&kyc_request, &e.filename, &base64::decode(&e.payload)?).await?;
    }

    context.site.outgoing_email_message().create(&person, &address, OutgoingEmailMessageKind::KycRequestReceived).await.unwrap();

    Ok(KycRequest::db_to_graphql(kyc_request, false).await?)
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "An request on certos")]
pub struct KycRequest {
  id: i32,
  person_id: PersonId,
  org_id: i32,
  created_at: UtcDateTime,
  state: String,
  name: Option<String>,
  last_name: Option<String>,
  id_number: Option<String>,
  id_type: Option<String>,
  birthdate: Option<UtcDateTime>,
  nationality: Option<String>,
  country: Option<String>,
  job_title: Option<String>,
  legal_entity_name: Option<String>,
  legal_entity_country: Option<String>,
  legal_entity_registration: Option<String>,
  legal_entity_tax_id: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct KycRequestFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  state_eq: Option<String>,
}

#[rocket::async_trait]
impl Showable<kyc_request::KycRequest, KycRequestFilter> for KycRequest {
  fn sort_field_to_order_by(field: &str) -> Option<KycRequestOrderBy> {
    match field {
      "id" => Some(KycRequestOrderBy::Id),
      "state" => Some(KycRequestOrderBy::State),
      "createdAt" => Some(KycRequestOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: KycRequestFilter) -> SelectKycRequest {
    SelectKycRequest {
      org_id_eq: Some(org_id),
      id_in: f.ids,
      id_eq: f.id_eq,
      state_eq: f.state_eq,
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectKycRequest {
    SelectKycRequest { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: kyc_request::KycRequest, _with_payload: bool) -> MyResult<Self> {
    let a = d.attrs;
    Ok(KycRequest {
      id: a.id,
      person_id: a.person_id,
      org_id: a.org_id,
      created_at: a.created_at,
      state: a.state,
      name: a.name,
      last_name: a.last_name,
      id_number: a.id_number,
      id_type: a.id_type,
      birthdate: a.birthdate,
      nationality: a.nationality,
      country: a.country,
      job_title: a.job_title,
      legal_entity_name: a.legal_entity_name,
      legal_entity_country: a.legal_entity_country,
      legal_entity_registration: a.legal_entity_registration,
      legal_entity_tax_id: a.legal_entity_tax_id,
    })
  }
}
