use super::*;
use serde::{Deserialize, Serialize};
use models::{kyc_request, outgoing_email_message_kind::*};

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "This object allows us to submit a kyc request to be considered by a constata admin")]
#[serde(rename_all = "camelCase")]
pub struct KycRequestInput {
  #[graphql(description = "email to be registered by the person")]
  pub email: String,
  #[graphql(description = "boolean pointing out whether the email should be registered as private or could be public")]
  pub keep_private: bool,
  #[graphql(description = "Your first names")]
  pub name: String,
  #[graphql(description = "Your last names")]
  pub last_name: String,
  #[graphql(description = "Government or otherwise officially issued ID number")]
  pub id_number: Option<String>,
  #[graphql(description = "Type of the officially issued id. Ej: DNI")]
  pub id_type: Option<String>,
  #[graphql(description = "Date of birth")]
  pub birthdate: Option<UtcDateTime>,
  #[graphql(description = "Country of birth")]
  pub nationality: Option<String>,
  #[graphql(description = "Country you currently live in.")]
  pub country: Option<String>,
  #[graphql(description = "Your role, title or position in your company, if any.")]
  pub job_title: Option<String>,
  #[graphql(description = "Name of the company")]
  pub legal_entity_name: Option<String>,
  #[graphql(description = "Country where the company is based on, or where it has its HQ.")]
  pub legal_entity_country: Option<String>,
  #[graphql(description = "Company registration number in the required public registries, if any.")]
  pub legal_entity_registration: Option<String>,
  #[graphql(description = "Company tax identification number")]
  pub legal_entity_tax_id: Option<String>,
  #[graphql(description = "Your company LinkedIn page, if any.")]
  pub legal_entity_linkedin_id: Option<String>,
  #[graphql(description = "The files attached as proof for the details provided.")]
  pub evidence: Vec<KycRequestEvidenceInput>,
}

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "This object has a file that proves the user identity")]
#[serde(rename_all = "camelCase")]
pub struct KycRequestEvidenceInput {
  #[graphql(description = "name of the file used as evidence for this kyc request")]
  pub filename: String,
  #[graphql(description = "the file itself")]
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
#[graphql(description = "This object show us the information of a kyc request")]
pub struct KycRequest {
  #[graphql(description = "number identifying this kyc request")]
  id: i32,
  #[graphql(description = "id of the person that requested this kyc")]
  person_id: PersonId,
  #[graphql(description = "id of the organization to which this person belongs")]
  org_id: i32,
  #[graphql(description = "date in which this kyc request was created")]
  created_at: UtcDateTime,
  #[graphql(description = "state of the kyc request, can be 'Pending' or 'Proccesed'")]
  state: String,
  #[graphql(description = "Your first names")]
  name: Option<String>,
  #[graphql(description = "Your last names")]
  last_name: Option<String>,
  #[graphql(description = "Government or otherwise officially issued ID number")]
  id_number: Option<String>,
  #[graphql(description = "Type of the officially issued id. Ej: DNI")]
  id_type: Option<String>,
  #[graphql(description = "Date of birth")]
  birthdate: Option<UtcDateTime>,
  #[graphql(description = "Country of birth")]
  nationality: Option<String>,
  #[graphql(description = "Country you currently live in.")]
  country: Option<String>,
  #[graphql(description = "Your role, title or position in your company, if any.")]
  job_title: Option<String>,
  #[graphql(description = "Name of the company")]
  legal_entity_name: Option<String>,
  #[graphql(description = "Country where the company is based on, or where it has its HQ.")]
  legal_entity_country: Option<String>,
  #[graphql(description = "Company registration number in the required public registries, if any.")]
  legal_entity_registration: Option<String>,
  #[graphql(description = "Company tax identification number")]
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
