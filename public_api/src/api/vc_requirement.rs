use super::*;
use db::*;

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "VcRequirements are reusable rules to describe and validate the claims of credentials expected by a VcPrompt. These are set up by us taylored to your needs.")]
pub struct VcRequirement {
  #[graphql(description = "Unique identifier of this requirement")]
  id: i32,
  #[graphql(description = "A friendly name or short description of this requirements, for example: 'A StudentCredential issued this year, or any FacultyCredential.'")]
  name: String,
  #[graphql(description = "Whether this rules have been archived by the user.")]
  archived: bool,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, serde::Serialize, serde::Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct VcRequirementFilter {
  #[arg(long, help="Fetch a specific list of requirements by their ids", action=clap::ArgAction::Append)]
  ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific requirement by id")]
  id_eq: Option<i32>,
  #[arg(long, help="Filter where name contains this text")]
  name_like: Option<String>,
  #[arg(long, help="Filter requirements that are or aren't archived")]
  archived_eq: Option<bool>,
}

#[rocket::async_trait]
impl Showable<db::VcRequirement, VcRequirementFilter> for VcRequirement {
  fn sort_field_to_order_by(field: &str) -> Option<VcRequirementOrderBy> {
    match field {
      "id" => Some(VcRequirementOrderBy::Id),
      "name" => Some(VcRequirementOrderBy::Name),
      "createdAt" => Some(VcRequirementOrderBy::CreatedAt),
      "archived" => Some(VcRequirementOrderBy::Archived),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<VcRequirementFilter>) -> SelectVcRequirement {
    if let Some(f) = filter {
      SelectVcRequirement {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        name_ilike: into_like_search(f.name_like),
        archived_eq: f.archived_eq,
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectVcRequirement {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectVcRequirement {
    SelectVcRequirement { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::VcRequirement) -> ConstataResult<Self> {
    Ok(VcRequirement {
      id: d.attrs.id,
      name: d.attrs.name,
      archived: d.attrs.archived,
    })
  }
}
