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
    #[sqlx_model_hints(varchar, default)]
    vidchain_code: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    vidchain_jwt: Option<String>,
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

  pub async fn vidchain_url(&self) -> ConstataResult<String> {
    let state = self.access_token().await?.attrs.token;
    let settings = &self.state.settings.vidchain;
    let host = &settings.host;
    let redirect_uri = &settings.redirect_uri;
    let client_id = &settings.client_id;
    let nonce = self.attrs.id;

    Ok(format!("{host}/oauth2/auth?response_type=code&state={state}&redirect_uri={redirect_uri}&client_id={client_id}&scope=openid%20VerifiableCredential&nonce={nonce}"))
  }

  pub async fn resolve_with_vidchain_code(self, code: &str) -> ConstataResult<Self> {
    let settings = &self.state.settings.vidchain;
    let response = ureq::post(&format!("{}/oauth2/token", &settings.host))
      .timeout(std::time::Duration::new(5,0))
      .set("Accept", "application/json")
      .send_form(&[
        ("code", code),
        ("client_id", &settings.client_id),
        ("client_secret", &settings.client_secret),
        ("redirect_uri", &settings.redirect_uri),
        ("grant_type", "authorization_code"),
      ])?;

    self.resolve_with_vidchain_jwt(code, response.into_string()?).await
  }

  pub async fn resolve_with_vidchain_jwt(self, code: &str, jwt: String) -> ConstataResult<Self> {
    use ssi::jwk::JWK;

    let conf = &self.state.settings.vidchain;
    let key: JWK = serde_json::from_value(serde_json::json!({
      "kty": "RSA",
      "n": conf.rsa_pk_n,
      "e": "AQAB",
      "alg": "RS256",
      "kid": conf.expected_kid,
    }))?;

    let response: serde_json::Value = serde_json::from_str(&jwt)?;
    let presentation_jwt = response["id_token"].as_str().unwrap();
    let claims: serde_json::Value = ssi::jwt::decode_verify(presentation_jwt, &key)?;

    let (state, notes) = self.validate_requirements(claims).await?;

    Ok(
      self.update()
        .vidchain_code(Some(code.to_string()))
        .vidchain_jwt(Some(jwt))
        .finished_at(Some(Utc::now()))
        .state(state)
        .state_notes(notes)
        .save().await?
    )
  }

  async fn validate_requirements(&self, claims: serde_json::Value) -> ConstataResult<(VcRequestState, Option<String>)> {
    use serde_json::json;

    let did_ethr = &self.state.settings.vidchain.did_ethr;
    if claims["aud"] != json!{["constata"]} || claims["did"] != json!{did_ethr} {
      return Ok((VcRequestState::Rejected, Some("aud_must_be_constata_and_did_must_be_vid".to_string())));
    }

    let rules = self.vc_prompt().await?.requirement_rules().await?;

    let Some(credentials) = claims.pointer("/vp/verifiableCredential").and_then(|x| x.as_array() ) else {
      return Ok((VcRequestState::Rejected, Some("presentation_had_no_credentials".to_string())))
    };

    for required_set in &rules.acceptable_sets {
      if required_set.matches(&credentials) {
        return Ok((VcRequestState::Approved, None));
      }
    }

    return Ok((VcRequestState::Rejected, Some("presentation_does_not_meet_requirements".to_string())));
  }
}

