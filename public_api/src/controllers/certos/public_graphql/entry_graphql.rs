use super::*;
use serde::{Serialize, Deserialize};

#[derive(GraphQLObject)]
#[graphql(description = "An Entry represents a single certified Diploma, Proof of Attendance, or Badge, that is part of a larger Issuance. Each entry is certified separately, and has its own state. If you make several Issuances in parallel, you may run out of tokens, and some Entries will be certified while others will remain pending until you purchase the tokens.")]
pub struct Entry {
  #[graphql(description = "Unique identifier for this Entry, across all Issuances.")]
  id: i32,
  #[graphql(description = "Id of the Issuance this entry belongs to.")]
  issuance_id: i32,
  #[graphql(description = "Name of the issuance this entry belongs to, for convenience.")]
  issuance_name: String,
  #[graphql(description = "This entries position within the larger Issuance. When the issuance is created from a CSV, this will be the row number.")]
  row_number: i32,
  #[graphql(description = "The state of this entry. Can be 'received': We got this row's data and will attempt to create a document such as a diploma, proof of attendance, or badge with the given details; 'created': The document was created correctly, and you should sign it; 'signed': You have signed this document, it is up to constata to certify it now; 'completed': The document has been timestamped and certified by Constata, and emailed to the recipient if required. 'failed': A problem ocurred that prevented further processing of this Entry, this could happen between 'received' and 'created' if the provided data is malformed. A failure in one single Entry will abort the whole Issuance, and nothing will be certified.")]
  state: String,
  #[graphql(description = "Date in which this entry was received by Constata")]
  received_at: UtcDateTime,
  #[graphql(description = "Parameters used to create this particular entry. If the issuance was created from a CSV, these will be the row's data.")]
  params: String,
  #[graphql(description = "Errors found when moving this entry from 'received' to 'created', if any.")]
  errors: Option<String>,
  #[graphql(description = "ID of the document that this entry belongs to.")]
  document_id: Option<String>,
  #[graphql(description = "ID of the story that this entry belongs to.")]
  story_id: Option<i32>,
  #[graphql(description = "Boolean whether this entries admin link has been visited.")]
  admin_visited: bool,
  #[graphql(description = "When published, this is the visit count for the public page.")]
  public_visit_count: i32,
  #[graphql(description = "Boolean that determines whether an email should be sent for this entry.")]
  has_email_callback: bool,
  #[graphql(description = "Date when the email was sent, if it has already been sent.")]
  email_callback_sent_at: Option<UtcDateTime>,
  #[graphql(description = "The data payload for this entry.")]
  payload: Option<String>,
  #[graphql(description = "The administrative access url for the direct recipient of this entry. They can use it to download, view or share the document.")]
  admin_access_url: Option<String>,
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
      "receivedAt" => Some(EntryOrderBy::ReceivedAt),
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

    let document = d.document().await?;
    let story_id = if let Some(d) = document.as_ref() { Some(d.story().await?.attrs.id) } else { None };

    let mut admin_visited: bool = false;
    let mut public_visit_count: i32 = 0;
    if let Some(doc) = document { 
      if let Some(l) = doc.download_proof_link_scope().optional().await? {
        admin_visited = l.attrs.admin_visited;
        public_visit_count = l.attrs.public_visit_count;
      }
    }
    let admin_access_url = d.admin_access_url().await?;

    Ok(Entry {
      id: d.attrs.id,
      issuance_id: d.attrs.request_id,
      issuance_name: d.request().await?.attrs.name,
      row_number: d.attrs.row_number,
      state: d.attrs.state,
      received_at: d.attrs.received_at,
      params: d.attrs.params,
      errors: d.attrs.errors,
      document_id: d.attrs.document_id,
      has_email_callback,
      email_callback_sent_at,
      story_id,
      admin_visited,
      public_visit_count,
      payload,
      admin_access_url
    })
  }
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "SigningIteratorInput Object")]
#[serde(rename_all = "camelCase")]
pub struct SigningIteratorInput {
  #[graphql(description = "ID of the issuance to which this entry belongs.")]
  issuance_id: i32,
  #[graphql(description = "Number that identifies this entry.")]
  entry_id: Option<i32>,
  #[graphql(description = "Signature applied to the referenced entry.")]
  signature: Option<String>,
}

impl SigningIteratorInput {
  pub async fn sign(self, context: &Context) -> FieldResult<Option<Entry>> {
    let db_data = match (self.entry_id, self.signature) {
      (Some(i), Some(s)) => Some(request::EntrySignature::from_base64(i, &s)?),
      _ => None,
    };
    
    let Some(next_entry) = context.site.request()
      .select()
      .id_eq(&self.issuance_id)
      .org_id_eq(context.org_id())
      .one()
      .await?
      .in_created()?
      .signing_iterator(db_data)
      .await?
      else { return Ok(None) };
      
    Ok(Some(Entry::db_to_graphql(next_entry, true).await?))
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "Represents an HTML preview of the contents of an entry.")]
pub struct Preview{
  #[graphql(description = "The numerical identifier of the entry.")]
  pub id: i32,
  #[graphql(description = "The HTML formatted contents of the entry.")]
  pub html: String
}

