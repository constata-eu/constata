use super::*;


#[derive(GraphQLObject)]
#[graphql(description = "an acceptance of terms and conditions")]
pub struct TermsAcceptance {
  id: i32,
  org_id: i32,
  person_id: i32,
  url: String,
  created_at: Option<UtcDateTime>,
  accepted: Option<UtcDateTime>,
  bulletin_id: Option<i32>,
  hash: Option<String>,
  evidence: Option<String>,
  email_address: Option<String>,
  parked_count: i32,
  parked_tokens: i32,
  first_parked_date: Option<UtcDateTime>,
  last_parked_date: Option<UtcDateTime>,
  missing_tokens: i32,
  pending_invoice_link_url: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct TermsAcceptanceFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  accepted_is_set: Option<bool>,
}

#[rocket::async_trait]
impl Showable<terms_acceptance::TermsAcceptance, TermsAcceptanceFilter> for TermsAcceptance {
  fn sort_field_to_order_by(field: &str) -> Option<TermsAcceptanceOrderBy> {
    match field {
      "id" => Some(TermsAcceptanceOrderBy::Id),
      "personId" => Some(TermsAcceptanceOrderBy::PersonId),
      "orgId" => Some(TermsAcceptanceOrderBy::OrgId),
      "createdAt" => Some(TermsAcceptanceOrderBy::CreatedAt),
      "accepted" => Some(TermsAcceptanceOrderBy::Accepted),
      _ => None,
    }
  }

  fn filter_to_select(f: TermsAcceptanceFilter) -> SelectTermsAcceptance {
    SelectTermsAcceptance{
      id_in: f.ids,
      id_eq: f.id_eq,
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      accepted_is_set: f.accepted_is_set,
      ..Default::default()
    }
  }
  
  async fn db_to_graphql(d: terms_acceptance::TermsAcceptance ) -> MyResult<Self> {
    let person = d.person().await?;
    let org = d.org().await?;
    let email_address = person.email_address().await?;
    let account_state = org.account_state().await?;
    let (first_parked_date, last_parked_date) = org.date_first_and_last_document_parked().await;
    Ok(TermsAcceptance {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      url: d.full_url(),
      created_at: d.attrs.created_at,
      accepted: d.attrs.accepted,
      bulletin_id: d.attrs.bulletin_id,
      hash: d.attrs.hash,
      evidence: d.attrs.evidence.map(|a| String::from_utf8_lossy(&a).to_string()),
      email_address: email_address.map(|i| i.attrs.address),
      parked_count: account_state.parked_count.to_i32().unwrap_or(0),
      parked_tokens: account_state.parked_tokens.to_i32().unwrap_or(0),
      first_parked_date,
      last_parked_date,
      missing_tokens: account_state.missing.to_i32().unwrap_or(0),
      pending_invoice_link_url: account_state.pending_invoice_link_url,
    })
  }
}
