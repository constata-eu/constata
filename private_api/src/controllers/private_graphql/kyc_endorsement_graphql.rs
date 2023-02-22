use super::*;
use chrono::Utc;

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A full kyc request")]
#[serde(rename_all = "camelCase")]
pub struct KycEndorsementInput {
  pub name: Option<String>,
  pub last_name: Option<String>,
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
  pub evidence: Vec<KycEndorsementEvidenceInput>,
}

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A full kyc request")]
#[serde(rename_all = "camelCase")]
pub struct KycEndorsementEvidenceInput {
  pub filename: String,
  pub payload: String,
}

impl KycEndorsementInput {
  pub async fn process(self, context: &Context, person_id: i32) -> FieldResult<KycEndorsement> {
    let person = context.site.person().find(&person_id).await?;

    let kyc_endorsement = person.get_or_create_kyc_endorsement().await?;

    let updated_kyc_endorsement = kyc_endorsement.update()
      .name(self.name)
      .last_name(self.last_name)
      .id_number(self.id_number)
      .id_type(self.id_type)
      .birthdate(self.birthdate)
      .nationality(self.nationality)
      .country(self.country)
      .job_title(self.job_title)
      .legal_entity_name(self.legal_entity_name)
      .legal_entity_country(self.legal_entity_country)
      .legal_entity_registration(self.legal_entity_registration)
      .legal_entity_tax_id(self.legal_entity_tax_id)
      .legal_entity_linkedin_id(self.legal_entity_linkedin_id)
      .updated_at(Utc::now())
      .save().await?;

    let evidence = self.evidence.iter()
      .map(|x| base64::decode(&x.payload) )
      .collect::<Result<Vec<_>,_>>()?;

    updated_kyc_endorsement.story().await?.save_evidence_and_model_changes(
      evidence,
      Some(&updated_kyc_endorsement)
    ).await?;

    Ok(KycEndorsement::db_to_graphql(updated_kyc_endorsement).await?)
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "know your customer endorsement")]
pub struct KycEndorsement {
  id: i32,
  org_id: i32,
  person_id: i32,
  story_id: i32,
  created_at: UtcDateTime,
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
  legal_entity_linkedin_id: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct KycEndorsementFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  name_like: Option<String>,
  last_name_like: Option<String>,
  id_type_like: Option<String>,
  id_number_like: Option<String>,
  country_like: Option<String>,
  job_title_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<kyc_endorsement::KycEndorsement, KycEndorsementFilter> for KycEndorsement {
  fn sort_field_to_order_by(field: &str) -> Option<KycEndorsementOrderBy> {
    match field {
      "id" => Some(KycEndorsementOrderBy::Id),
      "personId" => Some(KycEndorsementOrderBy::PersonId),
      "orgId" => Some(KycEndorsementOrderBy::OrgId),
      "storyId" => Some(KycEndorsementOrderBy::StoryId),
      "name" => Some(KycEndorsementOrderBy::Name),
      "lastName" => Some(KycEndorsementOrderBy::LastName),
      "idNumber" => Some(KycEndorsementOrderBy::IdNumber),
      "createdAt" => Some(KycEndorsementOrderBy::CreatedAt),
      "country" => Some(KycEndorsementOrderBy::Country),
      _ => None,
    }
  }

  fn filter_to_select(f: KycEndorsementFilter) -> SelectKycEndorsement {
    SelectKycEndorsement{
      id_in: f.ids,
      id_eq: f.id_eq,
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      name_ilike: into_like_search(f.name_like),
      last_name_ilike: into_like_search(f.last_name_like),
      id_type_ilike: into_like_search(f.id_type_like),
      id_number_ilike: into_like_search(f.id_number_like),
      country_ilike: into_like_search(f.country_like),
      job_title_ilike: into_like_search(f.job_title_like),
      ..Default::default()
    }
  }
  
  async fn db_to_graphql(d: kyc_endorsement::KycEndorsement ) -> MyResult<Self> {
    Ok(KycEndorsement {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      story_id: d.attrs.story_id,
      created_at: d.attrs.created_at,
      name: d.attrs.name,
      last_name: d.attrs.last_name,
      id_number: d.attrs.id_number,
      id_type: d.attrs.id_type,
      birthdate: d.attrs.birthdate,
      nationality: d.attrs.nationality,
      country: d.attrs.country,
      job_title: d.attrs.job_title,
      legal_entity_name: d.attrs.legal_entity_name,
      legal_entity_country: d.attrs.legal_entity_country,
      legal_entity_registration: d.attrs.legal_entity_registration,
      legal_entity_tax_id: d.attrs.legal_entity_tax_id,
      legal_entity_linkedin_id: d.attrs.legal_entity_linkedin_id,
    })
  }
}
