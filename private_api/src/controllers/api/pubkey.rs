use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "A Pubkey")]
pub struct Pubkey {
  id: String,
  person_id: i32,
  org_id: i32,
  bulletin_id: i32,
  evidence_signature: String,
  hash: String,
  signature_hash: String,
  evidence_payload: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct PubkeyFilter {
  ids: Option<Vec<String>>,
  id_like: Option<String>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  bulletin_id_eq: Option<i32>,
}

#[rocket::async_trait]
impl Showable<db::Pubkey, PubkeyFilter> for Pubkey {
  fn sort_field_to_order_by(field: &str) -> Option<PubkeyOrderBy> {
    match field {
      "id" => Some(PubkeyOrderBy::Id),
      "personId" => Some(PubkeyOrderBy::PersonId),
      "orgId" => Some(PubkeyOrderBy::OrgId),
      "bulletinId" => Some(PubkeyOrderBy::BulletinId),
      _ => None,
    }
  }

  fn filter_to_select(f: PubkeyFilter) -> SelectPubkey {
    SelectPubkey{
      id_in: f.ids,
      id_ilike: into_like_search(f.id_like),
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      bulletin_id_eq: f.bulletin_id_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: db::Pubkey ) -> ConstataResult<Self> {
    Ok(Pubkey {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      bulletin_id: d.attrs.bulletin_id,
      hash: d.attrs.hash,
      evidence_signature: d.attrs.evidence_signature,
      signature_hash: d.attrs.signature_hash,
      evidence_payload: String::from_utf8_lossy(&d.attrs.evidence_payload).to_string(),
    })
  }
}
