use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "a request of person deletion")]
pub struct OrgDeletion {
  id: i32,
  org_id: i32,
  story_id: i32,
  started_at: UtcDateTime,
  reason: DeletionReason,
  description: String,
  completed: bool,
  approving_admin_user: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct OrgDeletionFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  story_id_eq: Option<i32>,
  reason_eq: Option<String>,
  completed_eq: Option<bool>,
  
}

#[rocket::async_trait]
impl Showable<db::OrgDeletion, OrgDeletionFilter> for OrgDeletion {
  fn sort_field_to_order_by(field: &str) -> Option<OrgDeletionOrderBy> {
    match field {
      "id" => Some(OrgDeletionOrderBy::Id),
      "orgId" => Some(OrgDeletionOrderBy::OrgId),
      "storyId" => Some(OrgDeletionOrderBy::StoryId),
      "reason" => Some(OrgDeletionOrderBy::Reason),
      "completed" => Some(OrgDeletionOrderBy::Completed),
      "startedAt" => Some(OrgDeletionOrderBy::StartedAt),
      _ => None,
    }
  }

  fn filter_to_select(f: OrgDeletionFilter) -> SelectOrgDeletion {
    let reason_eq = f.reason_eq.and_then(|s|{
      match s.as_str() {
        "UserRequest" => Some(DeletionReason::UserRequest),
        "ConstataDecision" => Some(DeletionReason::ConstataDecision),
        "Inactivity" => Some(DeletionReason::Inactivity),
        _ => None,
      }
    });
    
    SelectOrgDeletion{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: f.org_id_eq,
      story_id_eq: f.story_id_eq,
      reason_eq,
      completed_eq: f.completed_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: db::OrgDeletion ) -> ConstataResult<Self> {
    let approving_admin_user = d.state.admin_user()
      .find(d.attrs.approving_admin_user).await?
      .attrs.username;

    Ok(OrgDeletion {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      story_id: d.attrs.story_id,
      reason: d.attrs.reason,
      description: d.attrs.description,
      completed: d.attrs.completed,
      approving_admin_user,
      started_at: d.attrs.started_at,
    })
  }
}

impl OrgDeletion {
  pub async fn create_org_deletion(
    context: &Context, org_id: i32, reason: String, description: String, evidence: String,
  ) -> FieldResult<OrgDeletion> {
    let enum_reason = match reason.as_str() {
      "UserRequest" => DeletionReason::UserRequest,
      "ConstataDecision" => DeletionReason::ConstataDecision,
      _ => DeletionReason::Inactivity,
    };

    let evidence_bytes = base64::decode(evidence)?;
    let db_org_deletion = context.site.org_deletion()
        .delete_org(org_id, context.id, enum_reason, description, vec![&evidence_bytes])
        .await?;

    Ok(OrgDeletion::db_to_graphql(db_org_deletion).await?)
  }

  pub async fn physical_deletion(context: &Context, org_deletion_id: i32) -> FieldResult<OrgDeletion> {
    if context.role != AdminRole::SuperAdmin {
      return Err(field_error("401", "you don't have permission and you tried to hack the UI"));
    }

    let db_org_deletion = context.site.org_deletion().find(&org_deletion_id).await?;
    db_org_deletion.physical_delete().await?;
    Ok(OrgDeletion::db_to_graphql(db_org_deletion).await?)
  }
}
