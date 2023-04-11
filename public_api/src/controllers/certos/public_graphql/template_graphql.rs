use super::*;
pub use constata_lib::models::template_schema::TemplateSchema;

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
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
  schema: TemplateSchema,
  #[graphql(description = "A personalized message that the user can add to the email sent to the student when the entry is certified")]
  custom_message: Option<String>,
  #[graphql(description = "Stats: How many recipients viewed the admin link that was sent to them.")]
  admin_visited_count: i32,
  #[graphql(description = "Entry count for all issuances that used this template.")]
  entries_count: i32,
  #[graphql(description = "Stats: How many visits did the published entries in this Template get, collectively.")]
  public_visit_count: i32,   
  #[graphql(description = "Whether this template was archived by the user.")]
  archived: bool,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, serde::Serialize, serde::Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFilter {
  #[arg(long, help="Fetch a specific list of templates by their ids", action=clap::ArgAction::Append)]
  ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific template by id")]
  id_eq: Option<i32>,
  #[arg(long, help="Filter where name contains this text")]
  name_like: Option<String>,
  #[arg(long, help="Filter templates that are or aren't archived")]
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

  fn filter_to_select(org_id: i32, filter: Option<TemplateFilter>) -> SelectTemplate {
    if let Some(f) = filter {
      SelectTemplate {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        name_ilike: into_like_search(f.name_like),
        archived_eq: f.archived_eq,
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectTemplate {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectTemplate {
    SelectTemplate { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: template::Template, _with_payload: bool) -> MyResult<Self> {
    let mut admin_visited_count = 0;
    let mut entries_count = 0;
    let mut public_visit_count = 0;

    for r in d.request_vec().await? {
      for e in r.entry_vec().await? {
        if e.is_signed() || e.is_completed() {
          entries_count += 1;
        }
        let Some(doc) = e.document().await? else { continue; };
        let Some(l) = doc.download_proof_link_scope().optional().await? else { continue; };
        if l.attrs.admin_visited { admin_visited_count += 1 };
        public_visit_count += l.attrs.public_visit_count;
      }
    } 

    Ok(Template {
      id: d.attrs.id,
      name: d.attrs.name,
      kind: d.attrs.kind,
      created_at: d.attrs.created_at,
      schema: serde_json::from_str(&d.attrs.schema)?,
      custom_message: d.attrs.custom_message,
      admin_visited_count,
      entries_count,
      public_visit_count,
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
