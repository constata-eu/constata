use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "A story")]
pub struct Story {
  id: i32,
  org_id: i32,
  open_until: Option<UtcDateTime>,
  markers: String,
  private_markers: String,
  total_documents_count: i32,
  published_documents_count: i32,
}

#[derive(Clone, GraphQLInputObject)]
pub struct StoryFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  markers_eq: Option<String>,
}

#[rocket::async_trait]
impl Showable<db::Story, StoryFilter> for Story {
  fn sort_field_to_order_by(field: &str) -> Option<StoryOrderBy> {
    match field {
      "id" => Some(StoryOrderBy::Id),
      "orgId" => Some(StoryOrderBy::OrgId),
      "markers" => Some(StoryOrderBy::Markers),
      "openUntil" => Some(StoryOrderBy::OpenUntil),
      _ => None,
    }
  }

  fn filter_to_select(f: StoryFilter) -> SelectStory {
    SelectStory{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: f.org_id_eq,
      markers_eq: f.markers_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: db::Story ) -> ConstataResult<Self> {
    let total_documents_count = d.documents().await?.len() as i32;
    let published_documents_count = d.published_documents().await?.len() as i32;

    Ok(Story {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      markers: d.attrs.markers,
      private_markers: d.attrs.private_markers,
      open_until: d.attrs.open_until,
      total_documents_count,
      published_documents_count,
    })
  }
}
