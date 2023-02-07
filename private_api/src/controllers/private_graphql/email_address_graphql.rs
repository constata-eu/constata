use super::*;
use constata_lib::models::UtcDateTime;

#[derive(GraphQLObject)]
#[graphql(description = "An email")]
pub struct EmailAddress {
  id: i32,
  address: String,
  person_id: i32,
  org_id: i32,
  verified_at: Option<UtcDateTime>,
  bulletin_id: i32,
  evidence: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct EmailAddressFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  address_like: Option<String>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  bulletin_id_eq: Option<i32>,
}

#[rocket::async_trait]
impl Showable<email_address::EmailAddress, EmailAddressFilter> for EmailAddress {
  fn sort_field_to_order_by(field: &str) -> Option<EmailAddressOrderBy> {
    match field {
      "id" => Some(EmailAddressOrderBy::Id),
      "address" => Some(EmailAddressOrderBy::Address),
      "personId" => Some(EmailAddressOrderBy::PersonId),
      "orgId" => Some(EmailAddressOrderBy::OrgId),
      "bulletinId" => Some(EmailAddressOrderBy::BulletinId),
      _ => None,
    }
  }

  fn filter_to_select(f: EmailAddressFilter) -> SelectEmailAddress {
    SelectEmailAddress{
      id_in: f.ids,
      id_eq: f.id_eq,
      address_ilike: into_like_search(f.address_like),
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      bulletin_id_eq: f.bulletin_id_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: email_address::EmailAddress ) -> MyResult<Self> {
    Ok(EmailAddress {
      id: d.attrs.id,
      address: d.attrs.address,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      bulletin_id: d.attrs.bulletin_id,
      verified_at: d.attrs.verified_at,
      evidence: String::from_utf8_lossy(&d.attrs.evidence).to_string(),
    })
  }
}
