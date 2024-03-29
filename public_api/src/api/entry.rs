use super::*;
use db::*;

#[derive(Debug, GraphQLObject, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "An Entry represents a single certified Diploma, Proof of Attendance, or Badge, that is part of a larger Issuance. Each entry is certified separately, and has its own state. If you make several Issuances in parallel, you may run out of tokens, and some Entries will be certified while others will remain pending until you purchase the tokens.")]
pub struct Entry {
  #[graphql(description = "Unique identifier for this Entry, across all Issuances.")]
  pub id: i32,
  #[graphql(description = "Id of the Issuance this entry belongs to.")]
  pub issuance_id: i32,
  #[graphql(description = "Name of the issuance this entry belongs to, for convenience.")]
  pub issuance_name: String,
  #[graphql(description = "This entries position within the larger Issuance. When the issuance is created from a CSV, this will be the row number.")]
  pub row_number: i32,
  #[graphql(description = "The state of this entry. Can be 'received': We got this row's data and will attempt to create a document such as a diploma, proof of attendance, or badge with the given details; 'created': The document was created correctly, and you should sign it; 'signed': You have signed this document, it is up to constata to certify it now; 'completed': The document has been timestamped and certified by Constata, and emailed to the recipient if required. 'failed': A problem ocurred that prevented further processing of this Entry, this could happen between 'received' and 'created' if the provided data is malformed. A failure in one single Entry will abort the whole Issuance, and nothing will be certified.")]
  pub state: String,
  #[graphql(description = "Date in which this entry was received by Constata")]
  pub received_at: UtcDateTime,
  #[graphql(description = "Parameters used to create this particular entry. If the issuance was created from a CSV, these will be the row's data.")]
  pub params: String,
  #[graphql(description = "Errors found when moving this entry from 'received' to 'created', if any.")]
  pub errors: Option<String>,
  #[graphql(description = "ID of the document that this entry belongs to.")]
  pub document_id: Option<String>,
  #[graphql(description = "ID of the story that this entry belongs to.")]
  pub story_id: Option<i32>,
  #[graphql(description = "Boolean whether this entries admin link has been visited.")]
  pub admin_visited: bool,
  #[graphql(description = "When published, this is the visit count for the public page.")]
  pub public_visit_count: i32,
  #[graphql(description = "Boolean that determines whether an email should be sent for this entry.")]
  pub has_email_callback: bool,
  #[graphql(description = "Date when the email was sent, if it has already been sent.")]
  pub email_callback_sent_at: Option<UtcDateTime>,
  #[graphql(description = "The administrative access url for the direct recipient of this entry. They can use it to download, view or share the document.")]
  pub admin_access_url: Option<String>,
}

#[derive(Clone, Default, Debug, GraphQLInputObject, Serialize, Deserialize)]
#[derive(clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct EntryFilter {
  #[arg(long, help="Fetch a specific list of entries by their ids", action=clap::ArgAction::Append)]
  pub ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific entry by id")]
  pub id_eq: Option<i32>,
  #[arg(long, help="Filter entries that belong to a specific issuance")]
  pub issuance_id_eq: Option<i32>,
  #[arg(long, help="Filter by state: 'received', 'created', 'signed', 'completed', 'failed'")]
  pub state_eq: Option<String>,
  pub document_id_eq: Option<String>,
  #[arg(long, help="Filter entries where the params contain this text")]
  pub params_like: Option<String>,
}


#[rocket::async_trait]
impl Showable<db::Entry, EntryFilter> for Entry {
  fn sort_field_to_order_by(field: &str) -> Option<EntryOrderBy> {
    match field {
      "id" => Some(EntryOrderBy::Id),
      "documentId" => Some(EntryOrderBy::DocumentId),
      "issuanceId" => Some(EntryOrderBy::IssuanceId),
      "state" => Some(EntryOrderBy::State),
      "rowNumber" => Some(EntryOrderBy::RowNumber),
      "receivedAt" => Some(EntryOrderBy::ReceivedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<EntryFilter>) -> SelectEntry {
    if let Some(f) = filter {
      SelectEntry {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        document_id_eq: f.document_id_eq,
        issuance_id_eq: f.issuance_id_eq,
        state_eq: f.state_eq,
        params_ilike: into_like_search(f.params_like),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectEntry {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectEntry {
    SelectEntry { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::Entry) -> ConstataResult<Self> {
    let (has_email_callback, email_callback_sent_at) = match d.attrs.email_callback_id {
      Some(x) => (true, d.state.email_callback().find(&x).await?.attrs.sent_at),
      _ => (false, None)
    };

    let document = d.document().await?;
    let story_id = if let Some(d) = document.as_ref() { Some(d.story().await?.attrs.id) } else { None };

    let (admin_visited, public_visit_count, admin_access_url) = if let Some(l) = d.admin_access_link().await? {
      (l.attrs.admin_visited, l.attrs.public_visit_count, Some(l.safe_env_url().await?))
    } else {
      (false, 0, None)
    };

    Ok(Entry {
      id: d.attrs.id,
      issuance_id: d.attrs.issuance_id,
      issuance_name: d.issuance().await?.attrs.name,
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
      admin_access_url
    })
  }
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "SigningIteratorInput Object")]
#[serde(rename_all = "camelCase")]
pub struct SigningIteratorInput {
  #[graphql(description = "ID of the issuance to which this entry belongs.")]
  pub issuance_id: i32,
  #[graphql(description = "Number that identifies this entry.")]
  pub entry_id: Option<i32>,
  #[graphql(description = "Signature applied to the referenced entry.")]
  pub signature: Option<String>,
}

impl SigningIteratorInput {
  pub async fn sign(self, context: &Context) -> FieldResult<Option<UnsignedEntryPayload>> {
    let db_data = match (self.entry_id, self.signature) {
      (Some(i), Some(s)) => Some(EntrySignature::from_base64(i, &s)?),
      _ => None,
    };
    
    let Some(next_entry) = context.site.issuance()
      .select()
      .id_eq(&self.issuance_id)
      .org_id_eq(context.org_id())
      .one()
      .await?
      .in_created()?
      .signing_iterator(db_data)
      .await?
      else { return Ok(None) };
      
    Ok(Some(UnsignedEntryPayload::db_to_graphql(next_entry).await?))
  }
}

#[derive(Debug, GraphQLObject, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "Represents an HTML preview of the contents of an entry.")]
pub struct PreviewEntry{
  #[graphql(description = "The numerical identifier of the entry.")]
  pub id: i32,
  #[graphql(description = "The HTML formatted contents of the entry.")]
  pub html: String
}

#[derive(Debug, GraphQLObject, Deserialize, Serialize)]
#[serde_as]
#[serde(rename_all = "camelCase")]
#[graphql(description = "Contains the verifiable HTML for this certified entry.", scalar=GqlScalar)]
pub struct UnsignedEntryPayload {
  #[graphql(description = "The numerical identifier of the entry.")]
  pub id: i32,

  #[graphql(description = "The entry itself.")]
  pub entry: Entry,

  #[graphql(description = "The base64 encoded contents of this entry. It's always a zip file.")]
  #[serde(with = "Base64Standard")]
  pub bytes: Bytes,
}

impl UnsignedEntryPayload {
  pub async fn db_to_graphql(d: db::Entry) -> ConstataResult<Self> {
    let bytes = d.payload().await?;
    Ok(Self {
      id: d.attrs.id,
      entry: Entry::db_to_graphql(d).await?,
      bytes
    })
  }
}

#[derive(Debug, GraphQLObject, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "Contains the verifiable HTML for this certified entry.")]
pub struct EntryHtmlExport{
  #[graphql(description = "The numerical identifier of the entry.")]
  pub id: i32,
  #[graphql(description = "The entry itself.")]
  pub entry: Entry,
  #[graphql(description = "The verifiable HTML proof.")]
  pub verifiable_html: String
}

