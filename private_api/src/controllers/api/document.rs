use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "A document")]
pub struct Document {
  id: String,
  person_id: i32,
  org_id: i32,
  bulletin_id: Option<i32>,
  funded: bool,
  funded_at: Option<UtcDateTime>,
  created_at: UtcDateTime,
  cost: i32,
  gift_id: Option<i32>,
  story_id: i32,
  sourced_from: DocumentSource,
  bulletin_state: Option<String>,
}

#[derive(Clone, GraphQLInputObject)]
pub struct DocumentFilter {
  ids: Option<Vec<String>>,
  id_like: Option<String>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  bulletin_id_eq: Option<i32>,
  author_id_eq: Option<i32>,
  story_id_eq: Option<i32>,
  funded_eq: Option<bool>,
  sourced_from_eq: Option<String>,
}

#[rocket::async_trait]
impl Showable<db::Document, DocumentFilter> for Document {
  fn sort_field_to_order_by(field: &str) -> Option<DocumentOrderBy> {
    match field {
      "id" => Some(DocumentOrderBy::Id),
      "orgId" => Some(DocumentOrderBy::OrgId),
      "personId" => Some(DocumentOrderBy::PersonId),
      "bulletinId" => Some(DocumentOrderBy::BulletinId),
      "storyId" => Some(DocumentOrderBy::StoryId),
      "funded" => Some(DocumentOrderBy::Funded),
      "fundedAt" => Some(DocumentOrderBy::FundedAt),
      "createdAt" => Some(DocumentOrderBy::CreatedAt),
      "cost" => Some(DocumentOrderBy::Cost),
      "sourcedFrom" => Some(DocumentOrderBy::SourcedFrom),
      _ => None,
    }
  }

  fn filter_to_select(f: DocumentFilter) -> SelectDocument {
    let sourced_from_eq = f.sourced_from_eq.and_then(|p|
      match p.as_str() {
        "Email" => Some(DocumentSource::Email),
        "Api" => Some(DocumentSource::Api),
        "Telegram" => Some(DocumentSource::Telegram),
        "Internal" => Some(DocumentSource::Internal),
        _ => None,
      }
    );

    SelectDocument{
      id_in: f.ids,
      id_ilike: into_like_search(f.id_like),
      org_id_eq: f.org_id_eq,
      bulletin_id_eq: f.bulletin_id_eq,
      story_id_eq: f.story_id_eq,
      person_id_eq: f.person_id_eq,
      funded_eq: f.funded_eq,
      sourced_from_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: db::Document ) -> ConstataResult<Self> {
    let bulletin_state: Option<String> = match &d.attrs.bulletin_id {
      Some(x) => Some(d.state.bulletin().find(x).await?.attrs.state),
      None => None,
    };

    Ok(Document {
      id: d.attrs.id,
      created_at: d.attrs.created_at,
      org_id: d.attrs.org_id,
      person_id: d.attrs.person_id,
      story_id: d.attrs.story_id,
      gift_id: d.attrs.gift_id,
      cost: d.attrs.cost.to_i32().unwrap_or(0),
      funded: d.attrs.funded,
      funded_at: d.attrs.funded_at,
      bulletin_id: d.attrs.bulletin_id,
      sourced_from: d.attrs.sourced_from,
      bulletin_state,
    })
  }
}
