use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "An template on certos")]
pub struct Template {
  id: i32,
  name: String,
  kind: TemplateKind,
  created_at: UtcDateTime,
  schema: Option<String>,
  custom_message: Option<String>,
}


#[derive(Clone, GraphQLInputObject, Debug)]
pub struct TemplateFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  name_like: Option<String>,
}


#[rocket::async_trait]
impl Showable<template::Template, TemplateFilter> for Template {
  fn sort_field_to_order_by(field: &str) -> Option<TemplateOrderBy> {
    match field {
      "id" => Some(TemplateOrderBy::Id),
      "name" => Some(TemplateOrderBy::Name),
      "createdAt" => Some(TemplateOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: TemplateFilter) -> SelectTemplate {
    SelectTemplate {
      id_in: f.ids,
      org_id_eq: Some(org_id),
      id_eq: f.id_eq,
      name_ilike: into_like_search(f.name_like),
      deletion_id_is_set: Some(false),
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectTemplate {
    SelectTemplate { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: template::Template, _with_payload: bool) -> MyResult<Self> {
    Ok(Template {
      id: d.attrs.id,
      name: d.attrs.name,
      kind: d.attrs.kind,
      created_at: d.attrs.created_at,
      schema: d.attrs.schema,
      custom_message: d.attrs.custom_message,
    })
  }
}
