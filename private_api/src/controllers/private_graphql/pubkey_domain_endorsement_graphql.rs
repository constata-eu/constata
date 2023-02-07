use super::*;


#[derive(GraphQLObject)]
#[graphql(description = "a domain endorsement")]
pub struct PubkeyDomainEndorsement {
  id: i32,
  org_id: i32,
  person_id: i32,
  state: String,
  pubkey_id: String,
  domain: String,
  request_signature: String,
  attempts: i32,
  next_attempt: UtcDateTime,
  attempts_log: String,
  bulletin_id: Option<i32>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct PubkeyDomainEndorsementFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  state_eq: Option<String>,
  pubkey_id_like: Option<String>,
  domain_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<pubkey_domain_endorsement::PubkeyDomainEndorsement, PubkeyDomainEndorsementFilter> for PubkeyDomainEndorsement {
  fn sort_field_to_order_by(field: &str) -> Option<PubkeyDomainEndorsementOrderBy> {
    match field {
      "id" => Some(PubkeyDomainEndorsementOrderBy::Id),
      "personId" => Some(PubkeyDomainEndorsementOrderBy::PersonId),
      "orgId" => Some(PubkeyDomainEndorsementOrderBy::OrgId),
      "state" => Some(PubkeyDomainEndorsementOrderBy::State),
      "pubkeyId" => Some(PubkeyDomainEndorsementOrderBy::PubkeyId),
      "domain" => Some(PubkeyDomainEndorsementOrderBy::Domain),
      _ => None,
    }
  }

  fn filter_to_select(f: PubkeyDomainEndorsementFilter) -> SelectPubkeyDomainEndorsement {
    SelectPubkeyDomainEndorsement{
      id_in: f.ids,
      id_eq: f.id_eq,
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      state_eq: f.state_eq,
      pubkey_id_ilike: into_like_search(f.pubkey_id_like),
      domain_ilike: into_like_search(f.domain_like),
      ..Default::default()
    }
  }
  
  async fn db_to_graphql(d: pubkey_domain_endorsement::PubkeyDomainEndorsement ) -> MyResult<Self> {
    Ok(PubkeyDomainEndorsement {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      state: d.attrs.state,
      pubkey_id: d.attrs.pubkey_id,
      domain: d.attrs.domain,
      request_signature: d.attrs.request_signature,
      attempts: d.attrs.attempts,
      next_attempt: d.attrs.next_attempt,
      attempts_log: d.attrs.attempts_log,
      bulletin_id: d.attrs.bulletin_id,
    })
  }
}
