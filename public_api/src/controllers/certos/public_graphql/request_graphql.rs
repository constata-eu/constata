use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "An request on certos")]
pub struct Request {
  id: i32,
  template_id: i32,
  template_name: String,
  template_kind: TemplateKind,
  state: String,
  name: String,
  created_at: UtcDateTime,
  errors: Option<String>,
  tokens_needed: Option<i32>,
  entries: Vec<Vec<String>>,
}

#[derive(GraphQLObject)]
#[graphql(description = "A CSV export of an Issuance")]
pub struct IssuanceExport {
  pub id: i32,
  pub csv: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct RequestFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  template_id_eq: Option<i32>,
  state_eq: Option<String>,
  name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<request::Request, RequestFilter> for Request {
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

  fn filter_to_select(org_id: i32, f: RequestFilter) -> SelectRequest {
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
    let tokens_needed = if d.is_created() {
      Some(d.in_created()?.tokens_needed().await?)
    } else {
      None
    };
    let mut entries = vec![];
    for entry in db_entries {
      entries.push(vec![
        entry.attrs.id.to_string(),
        entry.attrs.state,
        entry.attrs.document_id.unwrap_or_else(|| "".to_string()),
        entry.attrs.email_callback_id.map(|i| i.to_string()).unwrap_or_else(|| "".to_string())
      ]);
    }

    Ok(Request {
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
