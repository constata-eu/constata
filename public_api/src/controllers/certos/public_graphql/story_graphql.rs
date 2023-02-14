use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "Story Object: {
  id: number identifying this story,
  org_id: id of the organization to which this story belongs,
  open_until: expiration date to add documents to this story if any,
  private_markers: markers used in this story by the user,
  source: source from which the document was received,
  issuance_id: the issuance id if belongs to an issuance,
  issuance_name: the issuance name if belongs to an issuance,
  total_documents_count: amount of documents that this story has,
  published_documents_count: amount of certified documents that this story has,
  has_accepted_docs: boolean pointing out if this story has accepted documents waiting to be certified,
  is_terms_accepted: boolean pointing out if the user accepted the terms & conditions,
  download_proof_link: download proof link to visualize, download or publish the certificate if at least one document of the story is certified,
}")]
pub struct Story {
  id: i32,
  org_id: i32,
  open_until: Option<UtcDateTime>,
  private_markers: String,
  source: DocumentSource,
  issuance_id: Option<i32>,
  issuance_name: Option<String>,
  total_documents_count: i32,
  published_documents_count: i32,
  has_accepted_docs: bool,
  is_terms_accepted: bool,
  download_proof_link: Option<String>,
}

#[derive(Clone, GraphQLInputObject)]
pub struct StoryFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  private_markers_like: Option<String>,
}


#[rocket::async_trait]
impl Showable<story::Story, StoryFilter> for Story {
  fn sort_field_to_order_by(field: &str) -> Option<StoryOrderBy> {
    match field {
      "id" => Some(StoryOrderBy::Id),
      "privateMarkers" => Some(StoryOrderBy::PrivateMarkers),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: StoryFilter) -> SelectStory {
    SelectStory{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: Some(org_id),
      private_markers_like: into_like_search(f.private_markers_like),
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectStory {
    SelectStory { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: story::Story, _: bool) -> MyResult<Self> {
    let documents = d.documents().await?;
    let published_documents_count = d.published_documents().await?.len() as i32;
    let download_proof_link = d.create_download_proof_link(30).await?;
    let has_accepted_docs = d.has_accepted_docs().await?;
    let terms_acceptance = d.org().await?.get_or_create_terms_acceptance().await?;
    let is_terms_accepted = terms_acceptance.attrs.accepted.is_some();
    let entry_vec = documents[0].entry_vec().await?;
    let mut request_id = None;
    let mut request_name = None;
    if entry_vec.len() > 0 {
      let request = entry_vec[0].request().await?;
      request_id = Some(request.attrs.id);
      request_name = Some(request.attrs.name);
    }

    Ok(Story {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      private_markers: d.attrs.private_markers,
      open_until: d.attrs.open_until,
      source: documents[0].attrs.sourced_from,
      issuance_id: request_id,
      issuance_name: request_name,
      total_documents_count: documents.len() as i32,
      published_documents_count,
      has_accepted_docs,
      is_terms_accepted,
      download_proof_link,
    })
  }
}
