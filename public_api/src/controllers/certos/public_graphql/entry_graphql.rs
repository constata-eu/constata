use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "An entry on certos")]
pub struct Entry {
  id: i32,
  request_id: i32,
  request_name: String,
  row_number: i32,
  state: String,
  created_at: UtcDateTime,
  params: String,
  errors: Option<String>,
  document_id: Option<String>,
  story_id: Option<i32>,
  admin_visited: bool,
  public_visit_count: i32,
  has_email_callback: bool,
  email_callback_sent_at: Option<UtcDateTime>,
  payload: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct EntryFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  request_id_eq: Option<i32>,
  state_eq: Option<String>,
  document_id_eq: Option<String>,
  params_like: Option<String>,
}


#[rocket::async_trait]
impl Showable<entry::Entry, EntryFilter> for Entry {
  fn sort_field_to_order_by(field: &str) -> Option<EntryOrderBy> {
    match field {
      "id" => Some(EntryOrderBy::Id),
      "documentId" => Some(EntryOrderBy::DocumentId),
      "requestId" => Some(EntryOrderBy::RequestId),
      "state" => Some(EntryOrderBy::State),
      "rowNumber" => Some(EntryOrderBy::RowNumber),
      "createdAt" => Some(EntryOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: EntryFilter) -> SelectEntry {
    SelectEntry {
      id_in: f.ids,
      org_id_eq: Some(org_id),
      id_eq: f.id_eq,
      document_id_eq: f.document_id_eq,
      request_id_eq: f.request_id_eq,
      state_eq: f.state_eq,
      params_ilike: into_like_search(f.params_like),
      deletion_id_is_set: Some(false),
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectEntry {
    SelectEntry { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: entry::Entry, with_payload: bool) -> MyResult<Self> {
    let (has_email_callback, email_callback_sent_at) = match d.attrs.email_callback_id {
      Some(x) => (true, d.state.email_callback().find(&x).await?.attrs.sent_at),
      _ => (false, None)
    };

    let payload = if with_payload {
      Some(base64::encode(&d.payload().await?))
    } else {
      None
    };

    let document = d.document().await?;
      
    let mut story_id: Option<i32> = None;
    let mut admin_visited: bool = false;
    let mut public_visit_count: i32 = 0;

    if let Some(doc) = document { 
      story_id = Some(*doc.story_id());
      if let Some(l) = doc.download_proof_link_scope().optional().await? {
        admin_visited = l.attrs.admin_visited;
        public_visit_count = l.attrs.public_visit_count;
      }
    }
   
    Ok(Entry {
      id: d.attrs.id,
      request_id: d.attrs.request_id,
      request_name: d.request().await?.attrs.name,
      row_number: d.attrs.row_number,
      state: d.attrs.state,
      created_at: d.attrs.created_at,
      params: d.attrs.params,
      errors: d.attrs.errors,
      document_id: d.attrs.document_id,
      has_email_callback,
      email_callback_sent_at,
      story_id,
      admin_visited,
      public_visit_count,
      payload,
    })
  }
}

impl Entry {
  pub async fn signing_iterator(
    context: &Context, id: i32, entry_id: Option<i32>, signature: Option<String>
  ) -> FieldResult<Option<Entry>> {
    let db_data = match (entry_id, signature) {
      (Some(i), Some(s)) => Some(request::EntrySignature::from_base64(i, &s)?),
      _ => None,
    };
    
    let db_entry = context.site.request()
      .select()
      .id_eq(&id)
      .org_id_eq(context.org_id())
      .one()
      .await?
      .in_created()?
      .signing_iterator(db_data)
      .await?;
      
    let next_entry = match db_entry {
      Some(e) => Some(Entry::db_to_graphql(e, true).await?),
      None => None,
    };
      
    Ok(next_entry)
  }
}
