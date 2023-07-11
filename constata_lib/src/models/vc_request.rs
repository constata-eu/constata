use super::*;
use url::Url;
use futures_util::{future, pin_mut, StreamExt, SinkExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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
    vidchain_url: Option<String>,
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

#[derive(Debug, serde::Deserialize)]
struct VidchainValidationResult {
  validationResult: bool,
  did: String,
  jwt: String,
}

impl VcRequestHub {
  // In order to scan with VidWallet, we need to keep a websocket open to the vidchain / vidconnect site.
  // Vidconnect QR's expire every few seconds, and the user may scan a QR code and take too long to submit it,
  // which would result in an error, and needing to scan again.
  // To minimize chances of that, we use a sliding window with two valid websocket sessions for the same VcRequest.
  pub async fn wait_for_request_scans(&self) {
    let lock: Arc<RwLock<HashMap<i32, HashSet<UtcDateTime>>>> = Arc::new(RwLock::new(HashMap::new()));

    loop {
      let pending = self
        .select()
        .state_eq(VcRequestState::Pending)
        .all().await
        .unwrap().into_iter();

      for r in pending {
        let id = r.attrs.id;
        let now = Utc::now();
        let mut running = lock.write().await;

        if let Some(mut sessions) = running.get_mut(&id) {
          if sessions.len() >= 2 { continue; }

          // Sessions end by themselves, we don't start a new session unless the previous one
          // is at least 3 minutes old.
          let session_is_old = sessions.iter().next()
            .map(|i| i > &(now - chrono::Duration::minutes(3)) ) 
            .unwrap_or(false);
          if sessions.len() == 1 && session_is_old { continue; }

          sessions.insert(now.clone());
        } else {
          running.insert(id, HashSet::from_iter(vec![now.clone()].into_iter()));
        }

        let inner_lock = Arc::clone(&lock);
        tokio::spawn(async move {
          match r.request_on_vidchain().await {
            Err(e) => println!("Error processing vc_request {}: {} ", id, e),
            Ok(_) => println!("Processed vc_request {}", id),
          }
          inner_lock.write().await.entry(id).and_modify(|sessions|{ sessions.remove(&now); });
        });
      }
      tokio::time::sleep(Duration::from_millis(10)).await;
    }

  }
}

impl VcRequest {
  pub async fn submit_presentation(self, vc_presentation: String) -> sqlx::Result<Self> {
    self.update().vc_presentation(Some(vc_presentation)).save().await
  }

  pub async fn request_on_vidchain(self) -> ConstataResult<()> {
    let id = self.attrs.id.clone();

    let state = &self.access_token().await?.attrs.token;
    let settings = &self.state.settings.vidchain;
    let host = &settings.host;
    let redirect_uri = &settings.redirect_uri;
    let client_id = &settings.client_id;
    let nonce = self.attrs.id;
    let scope = self.vc_prompt().await?.requirement_rules().await?.vidchain_scope(&settings.enabled_scopes);
    let vidconnect_url = format!("{host}/oauth2/auth?response_type=code&state={state}&redirect_uri={redirect_uri}&client_id={client_id}&scope=openid%20{scope}&nonce={nonce}");
    let websocket = "wss://staging.vidchain.net/socket.io/?EIO=4&transport=websocket";

    let response = ureq::builder().redirects(0).build().get(&vidconnect_url).call()?;
    let redirect = Url::parse(response.header("location").unwrap()).unwrap();
    let login_challenge = redirect.query_pairs().filter(|(k,_)| k == "login_challenge").next().expect("login challenge").1;

    let (mut ws_stream, _) = connect_async(Url::parse(websocket).unwrap()).await.expect("Failed to connect");

    // Vidchain websockets send heartbeats every ~20 seconds.
    // QR codes expire at ~10 heartbeats, and show an error on vidwallet.
    // We reduce chances of that by closing this websocket session earlier, forcing the
    // calling process to open a new websocket session.
    let mut keep_alive = 8;

    while let Some(msg) = ws_stream.next().await {
      match msg {
        Ok(Message::Text(content)) => {
          if content.starts_with("0{") {
            ws_stream.send(Message::Text("40".into())).await.unwrap();
          } else if content.starts_with("40{\"sid\":") {
            ws_stream.send(Message::Text(
              format!(r#"42["signIn",{{"clientUriRedirect":"","challenge":"{login_challenge}","client_name":"{client_id}","scope":"openid,{scope}","isMobile":false}}]"#)
            )).await.unwrap();
          } else if content.starts_with("42[\"signIn\"") {
            // Do nothing
          } else if content.starts_with("42[\"printQR\"") {
            let json: serde_json::Value = serde_json::from_str(content.strip_prefix("42").unwrap()).unwrap();

            let siop_uri = json.pointer("/1/siopUri").unwrap().as_str().unwrap();

            let qr_uri = siop_uri.strip_prefix("vidchain://did-auth?").unwrap();
            self.clone().update().vidchain_url(Some(qr_uri.to_string())).save().await?;
          } else if content == "2" {
            if keep_alive > 0 {
              ws_stream.send(Message::Text("3".into())).await.unwrap();
              keep_alive -= 1;
            } else {
              ws_stream.close(None).await.unwrap();
            }
          } else if content.starts_with("42[\"signInResponse\"") {
            let sign_in_response = content.strip_prefix("42").unwrap();
            let json: serde_json::Value = serde_json::from_str(sign_in_response).unwrap();
            let result: VidchainValidationResult = serde_json::from_str(
              json.pointer("/1").expect("tuple with result at 1").as_str().unwrap()
            ).expect("validation result");
            self.clone().resolve_with_vidchain_jwt(&result.did, result.jwt).await?;
            ws_stream.close(None).await.unwrap();
          } else {
            ws_stream.close(None).await.unwrap();
          }
        },
        Ok(Message::Close(_)) => {},
        e => { println!("Unknown message: {:?}", e); }
      }
    }

    Ok(())
  }

  pub async fn resolve_with_vidchain_jwt(self, code: &str, presentation_jwt: String) -> ConstataResult<Self> {
    use ssi::jwk::JWK;

    let conf = &self.state.settings.vidchain;
    let key: JWK = serde_json::from_value(serde_json::json!({
      "kty": "RSA",
      "n": conf.rsa_pk_n,
      "e": "AQAB",
      "alg": "RS256",
      "kid": conf.expected_kid,
    }))?;

    let claims: serde_json::Value = ssi::jwt::decode_unverified(&presentation_jwt)?;
    let (state, notes) = self.validate_requirements(claims).await?;
    Ok(self.finish(state, notes, Some(presentation_jwt), Some(code.to_string())).await?)
  }

  pub async fn finish(mut self,
    state: VcRequestState,
    notes: Option<String>,
    vidchain_jwt: Option<String>,
    vidchain_code: Option<String>
  ) -> sqlx::Result<Self> {
    self.reload().await?;

    if self.attrs.state != VcRequestState::Pending {
      return Ok(self);
    }

    self.update()
      .vidchain_code(vidchain_code)
      .vidchain_jwt(vidchain_jwt)
      .finished_at(Some(Utc::now()))
      .state(state)
      .state_notes(notes)
      .save().await
  }

  async fn validate_requirements(&self, claims: serde_json::Value) -> ConstataResult<(VcRequestState, Option<String>)> {
    use serde_json::json;

    if claims["aud"] != json!{"https://staging.vidchain.net/siop/responses"} {
      return Ok((VcRequestState::Rejected, Some("aud_must_be_constata".to_string())));
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

