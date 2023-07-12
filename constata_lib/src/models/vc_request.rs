use super::*;
use url::Url;
use futures_util::{StreamExt, SinkExt};
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
    #[sqlx_model_hints(vc_request_state, default)]
    state: VcRequestState,
    #[sqlx_model_hints(varchar, default)]
    state_notes: Option<String>,
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
    vidchain_jwt: Option<String>,
    #[sqlx_model_hints(timestamptz, default)]
    last_active_at: Option<UtcDateTime>,
  },
  queries {
    pending_and_active_now("state = 'pending' AND (now() - last_active_at) < interval '2 minutes'"),
  },
  belongs_to {
    Org(org_id),
    VcPrompt(vc_prompt_id),
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

impl VcRequestHub {
  // In order to scan with VidWallet, we need to keep a websocket open to the vidchain / vidconnect site.
  // Vidconnect QR's expire every few seconds, and the user may scan a QR code and take too long to submit it,
  // which would result in an error, and needing to scan again.
  // To minimize chances of that, we use a sliding window with two valid websocket sessions for the same VcRequest.
  pub async fn wait_for_request_scans(&self) {
    let lock: Arc<RwLock<HashMap<i32, HashSet<UtcDateTime>>>> = Arc::new(RwLock::new(HashMap::new()));

    loop {
      let Ok(pending) = self.pending_and_active_now().all().await else {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        continue;
      };

      for r in pending {
        let id = r.attrs.id;
        let now = Utc::now();
        let mut running = lock.write().await;

        if let Some(sessions) = running.get_mut(&id) {
          if sessions.len() >= 2 { continue; }

          // Sessions end by themselves.
          // We don't start a new session unless the previous one is at least 3 minutes old.
          let session_is_old = sessions.iter().next()
            .map(|i| i > &(now - chrono::Duration::minutes(3)) ) 
            .unwrap_or(false);
          if sessions.len() == 1 && session_is_old { continue; }

          sessions.insert(now.clone());
        } else {
          running.insert(id, HashSet::from_iter(vec![now.clone()].into_iter()));
        }

        let inner_lock = Arc::clone(&lock);
        println!("Spawning task for {}", id);
        dbg!(&running);
        tokio::spawn(async move {
          match r.request_on_vidchain(8).await {
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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VidResult { validation_result: bool, did: String, jwt: String, }

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VidQrUri { siop_uri: String, }

impl VcRequest {
  pub async fn set_active(self) -> sqlx::Result<Self> {
    self.update().last_active_at(Some(Utc::now())).save().await
  }

  pub async fn request_on_vidchain(self, mut keep_alive: i32) -> ConstataResult<()> {
    let settings = &self.state.settings.vidchain;
    let client_id = &settings.client_id;
    let scope = self.vc_prompt().await?.requirement_rules().await?.vidchain_scope(&settings.enabled_scopes);

    let Some(login_challenge) = self.get_vidchain_login_challenge(&scope, client_id) else {
      return Err(Error::third_party("vidconnect", "could_not_retrieve_login_challenge"));
    };

    let (mut ws_stream, _) = connect_async(Url::parse("wss://staging.vidchain.net/socket.io/?EIO=4&transport=websocket")?).await?;

    while let Some(msg) = ws_stream.next().await {
      match msg {
        Ok(Message::Text(content)) => {
          if content.starts_with("0{") {
            ws_stream.send(Message::Text("40".into())).await?;
          } else if content.starts_with("40{\"sid\":") {
            ws_stream.send(Message::Text(
              format!(r#"42["signIn",{{"clientUriRedirect":"","challenge":"{login_challenge}","client_name":"{client_id}","scope":"openid,{scope}","isMobile":false}}]"#)
            )).await?;

          } else if content.starts_with("42[\"signIn\"") {
            // This message is expected, but we do nothing about it.
            
          } else if content.starts_with("42[\"printQR\"") {
            let url = self.parse_msg::<VidQrUri>(&content)
              .and_then(|siop| siop.siop_uri.strip_prefix("vidchain://did-auth?").map(str::to_string) )
              .ok_or_else(|| Error::third_party("vidconnect_invalid_printqr", &content))?;
            self.clone().update().vidchain_url(Some(url)).save().await?;

          } else if content == "2" {
            if keep_alive > 0 {
              ws_stream.send(Message::Text("3".into())).await?;
              keep_alive -= 1;
            } else {
              ws_stream.close(None).await?;
            }

          } else if content.starts_with("42[\"signInResponse\"") {
            let result = self.parse_msg::<String>(&content)
              .and_then(|s| serde_json::from_str::<VidResult>(&s).ok() )
              .ok_or_else(|| Error::third_party("vidconnect_invalid_sign_in_response", &content))?;

            if result.validation_result {
              self.clone().resolve_with_vidchain_jwt(&result.did, result.jwt).await?;
            }
            ws_stream.close(None).await?;
          } else {
            ws_stream.close(None).await?;
          }
        },
        Ok(Message::Close(_)) => {},
        e => { return Err(Error::third_party("vidconnect_websocket", &format!("{:?}", e))) }
      }
    }

    Ok(())
  }

  fn parse_msg<D: serde::de::DeserializeOwned>(&self, content: &str) -> Option<D> {
    Some(serde_json::from_str::<(String, D)>(content.strip_prefix("42")?).ok()?.1)
  }

  fn get_vidchain_login_challenge(&self, scope: &str, client_id: &str) -> Option<String> {
    let settings = &self.state.settings.vidchain;

    let url = format!(
      "{}/oauth2/auth?response_type=code&state=not_needed&redirect_uri={}&client_id={}&scope=openid%20{}&nonce={}",
      settings.host,
      settings.redirect_uri,
      client_id,
      scope,
      Utc::now().timestamp_millis()
    );

    ureq::builder().redirects(0).build().get(&url).call().ok()?
      .header("location").and_then(|x| Url::parse(x).ok() )?
      .query_pairs().filter(|(k,_)| k == "login_challenge").next()
      .map(|x| x.1.to_string() )
  }

  pub async fn resolve_with_vidchain_jwt(self, did: &str, presentation_jwt: String) -> ConstataResult<Self> {
    /*
    use ssi::jwk::JWK;

    let conf = &self.state.settings.vidchain;
    let key: JWK = serde_json::from_value(serde_json::json!({
      "kty": "RSA",
      "n": conf.rsa_pk_n,
      "e": "AQAB",
      "alg": "RS256",
      "kid": conf.expected_kid,
    }))?;
    */

    let claims: serde_json::Value = ssi::jwt::decode_unverified(&presentation_jwt)?;
    let (state, notes) = self.validate_requirements(claims).await?;
    Ok(self.finish(state, notes, Some(presentation_jwt), Some(did.to_string())).await?)
  }

  pub async fn finish(mut self,
    state: VcRequestState,
    notes: Option<String>,
    presentation_jwt: Option<String>,
    did: Option<String>
  ) -> sqlx::Result<Self> {
    self.reload().await?;

    if self.attrs.state != VcRequestState::Pending {
      return Ok(self);
    }

    self.update()
      .did(did)
      .vidchain_jwt(presentation_jwt)
      .finished_at(Some(Utc::now()))
      .state(state)
      .state_notes(notes)
      .save().await
  }

  async fn validate_requirements(&self, claims: serde_json::Value) -> ConstataResult<(VcRequestState, Option<String>)> {
    use serde_json::json;

    if claims["aud"] != json!{"https://staging.vidchain.net/siop/responses"} {
      return Ok((VcRequestState::Rejected, Some("invalid_audience".to_string())));
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
