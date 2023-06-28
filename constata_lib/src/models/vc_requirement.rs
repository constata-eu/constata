use super::*;

model!{
  state: Site,
  table: vc_requirements,
  struct VcRequirement {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(text)]
    name: String,
    #[sqlx_model_hints(text)]
    rules: String,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(bool, default)]
    archived: bool,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Org(org_id),
    OrgDeletion(deletion_id),
  }
}
