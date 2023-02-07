use super::*;


#[derive(GraphQLObject)]
#[graphql(description = "A bulletin")]
pub struct Bulletin {
  id: i32,
  state: String,
  started_at: UtcDateTime,
  hash: Option<String>,
  transaction: Option<String>,
  transaction_hash: Option<String>,
  block_hash: Option<String>,
  block_time: Option<UtcDateTime>,
  documents_count: i32,
  mempool_info_url: Option<String>,
  payload: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct BulletinFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  started_at_eq: Option<UtcDateTime>,
  state_eq: Option<String>,
  hash_eq: Option<String>,
  transaction_hash_eq: Option<String>,
  block_hash_eq: Option<String>,
}

#[rocket::async_trait]
impl Showable<bulletin::Bulletin, BulletinFilter> for Bulletin {
  fn sort_field_to_order_by(field: &str) -> Option<BulletinOrderBy> {
    match field {
      "id" => Some(BulletinOrderBy::Id),
      "startedAt" => Some(BulletinOrderBy::StartedAt),
      "state" => Some(BulletinOrderBy::State),
      _ => None,
    }
  }

  fn filter_to_select(f: BulletinFilter) -> SelectBulletin {
    SelectBulletin{
      id_in: f.ids,
      id_eq: f.id_eq,
      started_at_eq: f.started_at_eq,
      state_eq: f.state_eq,
      hash_eq: f.hash_eq,
      transaction_hash_eq: f.transaction_hash_eq,
      block_hash_eq: f.block_hash_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: bulletin::Bulletin ) -> MyResult<Self> {
    let payload = d.payload().await.ok();
    let mempool_info_url = d.mempool_info_url().await;

    Ok(Bulletin {
      documents_count: d.document_scope().count().await? as i32,
      id: d.attrs.id,
      state: d.attrs.state,
      started_at: d.attrs.started_at,
      hash:  d.attrs.hash,
      transaction: d.attrs.transaction,
      transaction_hash: d.attrs.transaction_hash,
      block_hash: d.attrs.block_hash,
      block_time: d.attrs.block_time,
      mempool_info_url,
      payload,
    })
  }
}