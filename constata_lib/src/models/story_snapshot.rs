use crate::models::{
  Site,
  model,
  UtcDateTime,
};

model!{
  state: Site,
  table: story_snapshots,
  struct StorySnapshot {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    story_id: i32,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(varchar)]
    hash: String,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: i32,
  }
}
