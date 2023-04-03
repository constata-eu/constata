use super::*;


#[derive(GraphQLObject, Clone, Debug)]
#[graphql(description = "A person using the system")]
pub struct Person {
  id: i32,
  org_id: i32,
  pubkey: Option<String>,
  email_address: Option<i32>,
  address: Option<String>,
  registration_date: Option<UtcDateTime>,
  is_terms_accepted: bool,
  terms_url: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct PersonFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
}

#[rocket::async_trait]
impl Showable<person::Person, PersonFilter> for Person {
  fn sort_field_to_order_by(field: &str) -> Option<PersonOrderBy> {
    match field {
      "id" => Some(PersonOrderBy::Id),
      "orgId" => Some(PersonOrderBy::OrgId),
      _ => None,
    }
  }

  fn filter_to_select(f: PersonFilter) -> SelectPerson {
    SelectPerson{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: f.org_id_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: person::Person ) -> MyResult<Self> {
    let email_address = d.email_address().await?;
    let pubkey = d.pubkey().await?;
    let terms_acceptance = d.get_or_create_terms_acceptance().await?;
    let terms_url = if terms_acceptance.attrs.accepted.is_none() {
      Some(terms_acceptance.full_url())
    } else {
      None
    };

    Ok(Person {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      pubkey: pubkey.map(|i| i.attrs.id),
      email_address: email_address.clone().map(|i| i.attrs.id),
      address: email_address.map(|i| i.attrs.address),
      registration_date: terms_acceptance.attrs.created_at,
      is_terms_accepted: terms_url.is_none(),
      terms_url,
    })
  }
}
