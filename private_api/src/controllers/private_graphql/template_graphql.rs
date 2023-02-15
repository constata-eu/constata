use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "An template on certos")]
pub struct Template {
  id: i32,
  name: String,
  person_id: i32,
  org_id: i32,
  created_at: UtcDateTime,
  kind: TemplateKind,
  schema: String,
  custom_message: Option<String>,
  og_title_override: Option<String>,
}


#[derive(Clone, GraphQLInputObject, Debug)]
pub struct TemplateFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  kind_eq: Option<String>,
  name_like: Option<String>,
}


#[rocket::async_trait]
impl Showable<template::Template, TemplateFilter> for Template {
  fn sort_field_to_order_by(field: &str) -> Option<TemplateOrderBy> {
    match field {
      "id" => Some(TemplateOrderBy::Id),
      "personId" => Some(TemplateOrderBy::PersonId),
      "orgId" => Some(TemplateOrderBy::OrgId),
      "name" => Some(TemplateOrderBy::Name),
      "kind" => Some(TemplateOrderBy::Kind),
      "createdAt" => Some(TemplateOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(f: TemplateFilter) -> SelectTemplate {
    let kind_eq = f.kind_eq.and_then(|p|
      match p.as_str() {
        "DIPLOMA" => Some(TemplateKind::Diploma),
        "ATTENDANCE" => Some(TemplateKind::Attendance),
        "INVITATION" => Some(TemplateKind::Invitation),
        _ => None,
      }
    );

    SelectTemplate {
      id_in: f.ids,
      id_eq: f.id_eq,
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      name_ilike: into_like_search(f.name_like),
      kind_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: template::Template) -> MyResult<Self> {
    Ok(Template {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      name: d.attrs.name,
      kind: d.attrs.kind,
      schema: d.attrs.schema,
      created_at: d.attrs.created_at,
      custom_message: d.attrs.custom_message,
      og_title_override: d.attrs.og_title_override,
    })
  }
}

impl Template {
  pub async fn create_template(
    context: &Context,
    org_id: i32,
    name: String,
    kind: String,
    evidence: String,
    schema: Option<String>,
    custom_message: Option<String>,
    og_title_override: Option<String>
  ) -> FieldResult<Template> {
    let evidence_bytes = base64::decode(evidence)?;
    let kind_enum = match kind.as_ref() {
      "DIPLOMA" => TemplateKind::Diploma,
      "ATTENDANCE" => TemplateKind::Attendance,
      _ => TemplateKind::Invitation,
    };
    let org = context.site.org().find(&org_id).await?;

    let db_template = context.site.template().insert(InsertTemplate{
      app_id: org.get_or_create_certos_app().await?.attrs.id,
      org_id,
      person_id: org.admin().await?.attrs.id,
      name,
      kind: kind_enum,
      custom_message,
      og_title_override,
      schema: schema.unwrap_or(serde_json::to_string(&kind_enum.default_schema())?),
      size_in_bytes: evidence_bytes.len() as i32,
    }).validate_and_save(&evidence_bytes).await?;
      
    Ok(Template::db_to_graphql(db_template).await?)
  }

  pub async fn update_template(
    context: &Context,
    id: i32,
    name: String,
    kind: String,
    schema: Option<String>,
    custom_message: Option<String>,
    og_title_override: Option<String>,
  ) -> FieldResult<Template> {

    let kind_enum = match kind.as_ref() {
      "DIPLOMA" => TemplateKind::Diploma,
      "ATTENDANCE" => TemplateKind::Attendance,
      _ => TemplateKind::Invitation,
    };
    let db_template = context.site.template().find(&id).await?
      .update()
      .name(name)
      .kind(kind_enum)
      .schema(schema.unwrap_or(serde_json::to_string(&kind_enum.default_schema())?))
      .custom_message(custom_message)
      .og_title_override(og_title_override)
      .save().await?;
      
    Ok(Template::db_to_graphql(db_template).await?)
  }
}
