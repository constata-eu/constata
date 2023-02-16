use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "An Issuance has many entries, which goes for certification once the user signs the issuance")]
pub struct Issuance {
  #[graphql(description = "number identifying the issuance")]
  id: i32,
  #[graphql(description = "id of the template linked to this issuance")]
  template_id: i32,
  #[graphql(description = "name of the template linked to this issuance")]
  template_name: String,
  #[graphql(description = "the kinds can be 'Diploma', 'Attendance' or 'Invitation")]
  template_kind: TemplateKind,
  #[graphql(description = "the states can be 'received', 'created', 'signed', 'completed' or 'failed'")]
  state: String,
  #[graphql(description = "the name of the issuance")]
  name: String,
  #[graphql(description = "date in which this issuance was created")]
  created_at: UtcDateTime,
  #[graphql(description = "errors that happened in the process of the issuance, if any")]
  errors: Option<String>,
  #[graphql(description = "amount of tokens that the user must buy to certify this issuance")]
  tokens_needed: Option<i32>,
  #[graphql(description = "entries that belong to this issuance")]
  entries: Vec<Entry>,
}

#[derive(GraphQLObject)]
#[graphql(description = "This object allows us to export the issuance information as a csv file")]
pub struct IssuanceExport {
  #[graphql(description = "number identifying the issuance")]
  pub id: i32,
  #[graphql(description = "a csv export of the issuance")]
  pub csv: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct IssuanceFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  template_id_eq: Option<i32>,
  state_eq: Option<String>,
  name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<request::Request, IssuanceFilter> for Issuance {
  fn sort_field_to_order_by(field: &str) -> Option<RequestOrderBy> {
    match field {
      "id" => Some(RequestOrderBy::Id),
      "templateId" => Some(RequestOrderBy::TemplateId),
      "state" => Some(RequestOrderBy::State),
      "name" => Some(RequestOrderBy::Name),
      "createdAt" => Some(RequestOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: IssuanceFilter) -> SelectRequest {
    SelectRequest {
      id_in: f.ids,
      org_id_eq: Some(org_id),
      id_eq: f.id_eq,
      template_id_eq: f.template_id_eq,
      state_eq: f.state_eq,
      state_ne: Some("failed".to_string()),
      name_ilike: into_like_search(f.name_like),
      deletion_id_is_set: Some(false),
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectRequest {
    SelectRequest { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: request::Request, _with_payload: bool) -> MyResult<Self> {
    let template = d.template().await?;
    let db_entries = d.entry_vec().await?;
    let tokens_needed = if d.is_created() { Some(d.in_created()?.tokens_needed().await?) }
    else { None };
    let mut entries = vec![];
    for entry in db_entries {
      entries.push(Entry::db_to_graphql(entry, false).await?);
    }
    Ok(Issuance {
      id: d.attrs.id,
      template_id: d.attrs.template_id,
      template_name: template.attrs.name,
      template_kind: template.attrs.kind,
      state: d.attrs.state,
      name: d.attrs.name,
      errors: d.attrs.errors,
      created_at: d.attrs.created_at,
      tokens_needed,
      entries,
    })
  }
}
