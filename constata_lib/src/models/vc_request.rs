use super::*;

model!{
  state: Site,
  table: vc_requests,
  struct VcRequest {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    vc_prompt_id: i32,
    #[sqlx_model_hints(int4)]
    access_token_id: i32,
    #[sqlx_model_hints(vc_request_state, default)]
    state: VcRequestState,
    #[sqlx_model_hints(varchar, default)]
    state_notes: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    vc_presentation: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    did: Option<String>,
    #[sqlx_model_hints(timestamptz, default)]
    started_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    finished_at: Option<UtcDateTime>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Org(org_id),
    VcPrompt(vc_prompt_id),
    AccessToken(access_token_id),
    OrgDeletion(deletion_id),
  }
}

#[derive(
  sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq,
  Serialize, juniper::GraphQLEnum, clap::ValueEnum
)]
#[sqlx(type_name = "vc_request_state", rename_all = "lowercase")]
pub enum VcRequestState {
  Pending,
  Approved,
  Rejected,
  Failed,
}

impl sqlx::postgres::PgHasArrayType for VcRequestState {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_vc_request_state")
  }
}

impl VcRequest {
  pub async fn submit_presentation(self, vc_presentation: String) -> sqlx::Result<Self> {
    self.update().vc_presentation(Some(vc_presentation)).save().await
  }

  pub async fn vidchain_url(&self) -> crate::error::Result<String> {
    let state = self.access_token().await?.attrs.token;
    let settings = &self.state.settings.vidchain;
    let host = &settings.host;
    let redirect_uri = &settings.redirect_uri;
    let client_id = &settings.client_id;
    let scope = self.vc_prompt().await?.attrs.rules;
    let nonce = self.attrs.id;

    Ok(format!("{host}/oauth2/auth?response_type=code&state={state}&redirect_uri={redirect_uri}&client_id={client_id}&scope=openid%20{scope}&nonce={nonce}"))
  }

  pub async fn resolve_with_vidchain_code(self, code: &str) -> sqlx::Result<Self> {
    self.update().state(VcRequestState::Approved).save().await
  }
}
