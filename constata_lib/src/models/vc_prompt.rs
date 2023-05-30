use super::*;

model!{
  state: Site,
  table: vc_prompts,
  struct VcPrompt {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(varchar)]
    name: String,
    #[sqlx_model_hints(int4)]
    access_token_id: i32,
    #[sqlx_model_hints(text)]
    rules: String,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    archived_at: Option<UtcDateTime>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Org(org_id),
    Person(person_id),
    AccessToken(access_token_id),
    OrgDeletion(deletion_id),
  },
  has_many {
    VcRequest(vc_prompt_id)
  }
}

impl VcPromptHub {
  pub async fn create(&self, person: &Person, name: &str, rules: &str) -> sqlx::Result<VcPrompt> {
    let access_token = self.state.access_token()
      .create(&person, AccessTokenKind::VcPrompt, None).await?;

    self.insert(InsertVcPrompt{
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      name: name.to_string(),
      access_token_id: access_token.attrs.id,
      rules: rules.to_string(),
    }).save().await
  }
}

impl VcPrompt {
  pub async fn archive(self) -> sqlx::Result<Self> {
    self.update().archived_at(Some(Utc::now())).save().await
  }

  pub async fn full_url(&self) -> crate::error::Result<String> {
    Ok(format!("{}/#/vc_prompt_kiosk/{}", &self.state.settings.url, self.access_token().await?.attrs.token))
  }

  pub async fn create_request(&self) -> sqlx::Result<VcRequest> {
    let access_token = self.state.access_token()
      .create(&self.org().await?.admin().await?, AccessTokenKind::VcRequest, None).await?;

    self.state.vc_request().insert(InsertVcRequest{
      org_id: self.attrs.org_id,
      vc_prompt_id: self.attrs.id,
      access_token_id: access_token.attrs.id
    }).save().await
  }
}
