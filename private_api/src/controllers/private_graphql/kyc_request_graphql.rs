use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "know your customer request")]
pub struct KycRequest {
  id: i32,
  org_id: i32,
  person_id: i32,
  kyc_endorsement_id: Option<i32>,
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
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  state_eq: Option<String>,
  name_like: Option<String>,
  last_name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<kyc_request::KycRequest, KycRequestFilter> for KycRequest {
  fn sort_field_to_order_by(field: &str) -> Option<KycRequestOrderBy> {
    match field {
      "id" => Some(KycRequestOrderBy::Id),
      "orgId" => Some(KycRequestOrderBy::OrgId),
      "personId" => Some(KycRequestOrderBy::PersonId),
      "createdAt" => Some(KycRequestOrderBy::CreatedAt),
      "state" => Some(KycRequestOrderBy::State),
      "name" => Some(KycRequestOrderBy::Name),
      "lastName" => Some(KycRequestOrderBy::LastName),
      _ => None,
    }
  }

  fn filter_to_select(f: KycRequestFilter) -> SelectKycRequest {
    SelectKycRequest{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: f.org_id_eq,
      person_id_eq: f.person_id_eq,
      state_eq: f.state_eq,
      name_ilike: into_like_search(f.name_like),
      last_name_ilike: into_like_search(f.last_name_like),
      ..Default::default()
    }
  }
  
  async fn db_to_graphql(d: kyc_request::KycRequest ) -> MyResult<Self> {
    let kyc_endorsement_id = d.state
      .person().find(d.person_id()).await?
      .kyc_endorsement().await?.map(|e| e.attrs.id);

    Ok(KycRequest {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      person_id: d.attrs.person_id,
      kyc_endorsement_id,
      created_at: d.attrs.created_at,
      state: d.attrs.state,
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
    })
  }
}

impl KycRequest {
  pub async fn update_kyc_request(context: &Context, id: i32, form: String) -> FieldResult<KycRequest> {
    let kyc_request_process_form: KycRequestProcessForm = serde_json::from_str(&form)?;
    let kyc_request = context.site.kyc_request().find(&id).await?
      .in_pending()?.process_update(kyc_request_process_form).await?;
    Ok(KycRequest::db_to_graphql(kyc_request).await?)
  }
}
