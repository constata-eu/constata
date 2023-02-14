use super::*;
use serde::{Serialize, Deserialize};

#[derive(GraphQLObject)]
#[graphql(description = "Entry Object")]
pub struct Entry {
  #[graphql(description = "number identifying an entry")]
  id: i32,
  #[graphql(description = "id of the issuance to which this entry belongs")]
  issuance_id: i32,
  #[graphql(description = "name of the issuance to which this entry belongs")]
  issuance_name: String,
  #[graphql(description = "entry number within the issuance")]
  row_number: i32,
  #[graphql(description = "the states can be 'created', 'signed', 'completed' or 'failed'")]
  state: String,
  #[graphql(description = "date in which this entry was created")]
  created_at: UtcDateTime,
  #[graphql(description = "parameters used to create this particular entry")]
  params: String,
  #[graphql(description = "errors that happened in the process of the entry, if any")]
  errors: Option<String>,
  #[graphql(description = "id of the document to which this entry belongs")]
  document_id: Option<String>,
  #[graphql(description = "id of the story to which this entry belongs")]
  story_id: Option<i32>,
  #[graphql(description = "boolean that tells us if an email should be sent for this entry")]
  has_email_callback: bool,
  #[graphql(description = "date the email was sent, if it already happened")]
  email_callback_sent_at: Option<UtcDateTime>,
  #[graphql(description = "the entry itself")]
  payload: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct EntryFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  issuance_id_eq: Option<i32>,
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
      "issuanceId" => Some(EntryOrderBy::RequestId),
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
      request_id_eq: f.issuance_id_eq,
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

    let document = match d.attrs.document_id.as_ref() {
      Some(x) => Some(d.state.document().find(x).await?),
      _ => None,
    };
    let story_id = match document {
      Some(x) => Some(d.state.story().find(x.attrs.story_id).await?.attrs.id),
      _ => None,
    };

    Ok(Entry {
      id: d.attrs.id,
      issuance_id: d.attrs.request_id,
      issuance_name: d.request().await?.attrs.name,
      row_number: d.attrs.row_number,
      state: d.attrs.state,
      created_at: d.attrs.created_at,
      params: d.attrs.params,
      errors: d.attrs.errors,
      document_id: d.attrs.document_id,
      has_email_callback,
      email_callback_sent_at,
      story_id,
      payload,
    })
  }
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "SigningIteratorInput Object")]
#[serde(rename_all = "camelCase")]
pub struct SigningIteratorInput {
  #[graphql(description = "id of the issuance to which this entry belongs")]
  issuance_id: i32,
  #[graphql(description = "number identifying an entry")]
  entry_id: Option<i32>,
  #[graphql(description = "the signature of the user's Bitcoin wallet that signs this entry")]
  signature: Option<String>,
}


impl SigningIteratorInput {
  pub async fn sign(self, context: &Context) -> FieldResult<Option<Entry>> {
    let db_data = match (self.entry_id, self.signature) {
      (Some(i), Some(s)) => Some(request::EntrySignature::from_base64(i, &s)?),
      _ => None,
    };
    
    let db_entry = context.site.request()
      .select()
      .id_eq(&self.issuance_id)
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
