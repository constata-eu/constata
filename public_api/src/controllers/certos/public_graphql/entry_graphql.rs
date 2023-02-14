use super::*;
use serde::{Serialize, Deserialize};

#[derive(GraphQLObject)]
#[graphql(description = "Entry Object: {
  id: number identifying an entry,
  issuance_id: id of the issuance to which this entry belongs,
  issuance_name: name of the issuance to which this entry belongs,
  row_number: entry number within the issuance,
  state: the states can be 'created', 'signed', 'completed' or 'failed',
  created_at: date in which this entry was created,
  params: parameters used to create this particular entry,
  errors: errors that happened in the process of the entry, if any,
  document_id: id of the document to which this entry belongs,
  story_id: id of the story to which this entry belongs,
  has_email_callback: boolean that tells us if an email should be sent for this entry,
  email_callback_sent_at: date the email was sent, if it already happened,
  payload: the entry itself,
}")]
pub struct Entry {
  id: i32,
  issuance_id: i32,
  issuance_name: String,
  row_number: i32,
  state: String,
  created_at: UtcDateTime,
  params: String,
  errors: Option<String>,
  document_id: Option<String>,
  story_id: Option<i32>,
  has_email_callback: bool,
  email_callback_sent_at: Option<UtcDateTime>,
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
#[graphql(description = "Signing Iterator Input Object: {
  issuance_id: id of the issuance to which this entry belongs,
  entry_id: number identifying an entry,
  signature: the signature of the user's Bitcoin wallet that signs this entry,
}")]
#[serde(rename_all = "camelCase")]
pub struct SigningIteratorInput {
  issuance_id: i32,
  entry_id: Option<i32>,
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
