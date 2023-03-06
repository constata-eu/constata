use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "A template that serves as a foundation for creating each Issuance Entry.")]
pub struct Template {
  #[graphql(description = "An identifier for this template")]
  id: i32,
  #[graphql(description = "The name of this template")]
  name: String,
  #[graphql(description = "The type of issuance that can be created from this template. Possible values are 'Diploma', 'Attendance', or 'Badge'.")]
  kind: TemplateKind,
  #[graphql(description = "The date when this template was created")]
  created_at: UtcDateTime,
  #[graphql(description = "The schema used to build the issuance entry")]
  schema: String,
  #[graphql(description = "A personalized message that the user can add to the email sent to the student when the entry is certified")]
  custom_message: Option<String>,
  archived: bool,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct TemplateFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  name_like: Option<String>,
  archived_eq: Option<bool>,
}

#[rocket::async_trait]
impl Showable<template::Template, TemplateFilter> for Template {
  fn sort_field_to_order_by(field: &str) -> Option<TemplateOrderBy> {
    match field {
      "id" => Some(TemplateOrderBy::Id),
      "name" => Some(TemplateOrderBy::Name),
      "createdAt" => Some(TemplateOrderBy::CreatedAt),
      "archived" => Some(TemplateOrderBy::Archived),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: TemplateFilter) -> SelectTemplate {
    SelectTemplate {
      id_in: f.ids,
      org_id_eq: Some(org_id),
      id_eq: f.id_eq,
      name_ilike: into_like_search(f.name_like),
      archived_eq: f.archived_eq,
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
      archived: d.attrs.archived,
    })
  }
}

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A template input")]
pub struct TemplateInput {
  pub id: i32,
  pub action: String,
}


impl TemplateInput {
  pub async fn update_template(&self, context: &Context) -> FieldResult<Template> {
    let template = context.site.template()
      .select().id_eq(&self.id).org_id_eq(context.org_id()).one().await?
      .update().archived(self.action == "archive").save().await?;

    Ok(Template::db_to_graphql(template, false).await?)
  }
}