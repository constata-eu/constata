use super::*;
pub use constata_lib::models::{self, SelectVcPrompt, VcPromptOrderBy};

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "A prompt managing the creation an validation of verifiable credential presentation requests.")]
pub struct VcPrompt {
  #[graphql(description = "Unique numeric identifier of this prompt")]
  id: i32,
  #[graphql(description = "A friendly, public name of this prompt.")]
  name: String,
  #[graphql(description = "The custom rules applied to the verifiable credentials. (Just the credential type for now.)")]
  rules: String,
  #[graphql(description = "The public url for this prompt, can be opened from any device with the link.")]
  full_url: String,
  #[graphql(description = "The date when this prompt was created.")]
  created_at: UtcDateTime,
  #[graphql(description = "Archived prompts cannot issue further VC presentation requests.")]
  archived_at: Option<UtcDateTime>,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, serde::Serialize, serde::Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct VcPromptFilter {
  #[arg(long, help="Fetch a specific list of prompts by their ids", action=clap::ArgAction::Append)]
  ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific prompt by id")]
  id_eq: Option<i32>,
  #[arg(long, help="Filter where name contains this text")]
  name_like: Option<String>,
  #[arg(long, help="Filter prompts that are or aren't archived")]
  archived_at_is_set: Option<bool>,
}

#[rocket::async_trait]
impl Showable<models::VcPrompt, VcPromptFilter> for VcPrompt {
  fn sort_field_to_order_by(field: &str) -> Option<VcPromptOrderBy> {
    match field {
      "id" => Some(VcPromptOrderBy::Id),
      "name" => Some(VcPromptOrderBy::Name),
      "createdAt" => Some(VcPromptOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<VcPromptFilter>) -> SelectVcPrompt {
    if let Some(f) = filter {
      SelectVcPrompt {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        name_ilike: into_like_search(f.name_like),
        archived_at_is_set: f.archived_at_is_set,
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectVcPrompt {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectVcPrompt {
    SelectVcPrompt { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: models::VcPrompt) -> ConstataResult<Self> {
    let full_url = d.full_url().await?;

    Ok(VcPrompt {
      id: d.attrs.id,
      name: d.attrs.name,
      full_url,
      rules: d.attrs.rules,
      created_at: d.attrs.created_at,
      archived_at: d.attrs.archived_at,
    })
  }
}

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "The input type for archiving a prompt.")]
#[serde(rename_all = "camelCase")]
pub struct CreateVcPromptInput {
  pub name: String,
  pub rules: String,
}

impl CreateVcPromptInput {
  pub async fn process(&self, context: &Context) -> FieldResult<VcPrompt> {
    let prompt = context.site.vc_prompt()
      .create(&context.person(), &self.name, &self.rules).await?;
    Ok(VcPrompt::db_to_graphql(prompt).await?)
  }
}

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "The input type for archiving a prompt.")]
#[serde(rename_all = "camelCase")]
pub struct UpdateVcPromptInput {
  pub id: i32,
  pub action: String,
}

impl UpdateVcPromptInput {
  pub async fn process(&self, context: &Context) -> FieldResult<VcPrompt> {
    let mut prompt = context.site.vc_prompt().select()
      .use_struct(VcPrompt::select_by_id(context.org_id(), self.id))
      .one().await?;

    if self.action == "archive" {
      prompt = prompt.archive().await?;
    }
    Ok(VcPrompt::db_to_graphql(prompt).await?)
  }
}
