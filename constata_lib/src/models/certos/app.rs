use crate::models::{
  model,
  Site,
};

model!{
  state: Site,
  table: certos_apps,
  struct App {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
  }
}
