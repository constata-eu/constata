use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "The public key of a Person is used to sign certificates created through Issuances and Attestations. It's also used to authenticate to this API.")]
pub struct Pubkey {
  #[graphql(description = "Unique ID of the pubkey, corresponds to a mainnet bitcoin address.")]
  id: String,
  person_id: i32,
  org_id: i32,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct PubkeyFilter {
  ids: Option<Vec<String>>,
  id_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<db::Pubkey, PubkeyFilter> for Pubkey {
  fn sort_field_to_order_by(field: &str) -> Option<PubkeyOrderBy> {
    match field {
      "id" => Some(PubkeyOrderBy::Id),
      "personId" => Some(PubkeyOrderBy::PersonId),
      "orgId" => Some(PubkeyOrderBy::OrgId),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<PubkeyFilter>) -> SelectPubkey {
    if let Some(f) = filter {
      SelectPubkey {
        id_in: f.ids,
        id_ilike: into_like_search(f.id_like),
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectPubkey {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: String) -> SelectPubkey {
    SelectPubkey { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::Pubkey) -> ConstataResult<Self> {
    Ok(Pubkey {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
    })
  }
}