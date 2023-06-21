use super::{
  *,
  attestation::Attestation,
};
use crate::signed_payload::SignedPayload;
use juniper::GraphQLEnum;
use duplicate::duplicate_item;

model!{
  state: Site,
  table: web_callbacks,
  struct WebCallback {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(web_callback_kind)]
    kind: WebCallbackKind,
    #[sqlx_model_hints(int4)]
    resource_id: i32,
    #[sqlx_model_hints(web_callback_state, default)]
    state: WebCallbackState,
    #[sqlx_model_hints(int4, default)]
    last_attempt_id: Option<i32>,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    next_attempt_on: UtcDateTime,
  },
  belongs_to {
    Org(org_id),
    WebCallbackAttempt(last_attempt_id),
  },
  has_many {
    WebCallbackAttempt(web_callback_id),
  }
}

model!{
  state: Site,
  table: web_callback_attempts,
  struct WebCallbackAttempt {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    web_callback_id: i32,
    #[sqlx_model_hints(timestamptz)]
    attempted_at: UtcDateTime,
    #[sqlx_model_hints(varchar)]
    url: String,
    #[sqlx_model_hints(web_callback_result_code)]
    result_code: WebCallbackResultCode,
    #[sqlx_model_hints(varchar)]
    result_text: String,
  },
  belongs_to {
    Org(org_id),
    WebCallback(web_callback_id),
  },
}

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum, clap::ValueEnum)]
#[sqlx(type_name = "web_callback_state", rename_all = "lowercase")]
#[serde(rename_all= "SCREAMING_SNAKE_CASE")]
pub enum WebCallbackState {
  Pending,
  Done,
  Failed,
}

impl sqlx::postgres::PgHasArrayType for WebCallbackState {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_web_callback_state")
  }
}

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum, clap::ValueEnum)]
#[sqlx(type_name = "web_callback_kind", rename_all = "snake_case")]
#[serde(rename_all= "SCREAMING_SNAKE_CASE")]
pub enum WebCallbackKind {
  AttestationDone,
}

impl sqlx::postgres::PgHasArrayType for WebCallbackKind {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_web_callback_kind")
  }
}

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum)]
#[sqlx(type_name = "web_callback_result_code", rename_all = "snake_case")]
#[serde(rename_all= "SCREAMING_SNAKE_CASE")]
pub enum WebCallbackResultCode {
  Ok,
  NetworkError,
  NoCallbacksUrlForOrg,
  NonSuccessResponse,
}

impl sqlx::postgres::PgHasArrayType for WebCallbackResultCode {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_web_callback_result_code")
  }
}

impl WebCallbackHub {
  pub async fn schedule_attestation_done(&self, attestation: &Attestation) -> ConstataResult<Option<Pending>> {
    let org = attestation.org().await?;
    if org.web_callbacks_url().is_none() {
      return Ok(None);
    }

    let existing = self.state.web_callback().select()
      .org_id_eq(&org.attrs.id)
      .kind_eq(&WebCallbackKind::AttestationDone)
      .resource_id_eq(&attestation.attrs.id)
      .state_eq(&WebCallbackState::Pending)
      .optional().await?;

    let callback = if let Some(callback) = existing {
      callback
    } else {
      self.insert(InsertWebCallback{
        org_id: org.attrs.id,
        kind: WebCallbackKind::AttestationDone,
        resource_id: attestation.attrs.id,
      }).save().await?
    };

    Ok(callback.in_pending().ok())
  }
}

impl WebCallback {
  pub fn flow(&self) -> Flow {
    match self.state() {
      WebCallbackState::Pending => Flow::Pending(Pending(self.clone())),
      WebCallbackState::Done => Flow::Done(Done(self.clone())),
      _ => Flow::Failed(Failed(self.clone())),
    }
  }

  pub async fn request_body(&self) -> ConstataResult<String> {
    let key = self.state.keyring()?
      .map(|o| o.private_key )
      .ok_or_else(|| Error::Internal("No key for signinig callbacks".to_string()))?;

    let content = match self.kind() {
      kind @ WebCallbackKind::AttestationDone => {
        let attestation = self.state.attestation().select().org_id_eq(self.org_id()).id_eq(self.resource_id()).one().await?;
        let resource = WebCallbackResource::AttestationDone(super::attestation::for_api::from_model(attestation).await?);
        WebCallbackContent{ kind: *kind, resource }
      },
    };

    let signed_payload = SignedPayload::create(
      serde_json::to_string(&content)?.as_bytes(),
      &key,
      self.state.settings.network
    );

    Ok(serde_json::to_string(&signed_payload)?)
  }
}

#[duplicate_item(
  in_state        is_state        state_variant                   state_struct;
  [ in_pending  ] [ is_pending  ] [ WebCallbackState::Pending   ] [ Pending   ];
  [ in_done     ] [ is_done     ] [ WebCallbackState::Done      ] [ Done      ];
  [ in_failed   ] [ is_failed   ] [ WebCallbackState::Failed    ] [ Failed    ];
)]
impl WebCallback {
  pub fn in_state(&self) -> ConstataResult<state_struct> {
    self.flow().in_state()
  }

  pub fn is_state(&self) -> bool {
    matches!(self.state(), state_variant)
  }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  // The web callback has been scheduled and may be attempted.
  Pending(Pending),
  // The web callback had too many retries.
  Failed(Failed),
  // The web callback was sent and received correctly.
  Done(Done),
}

#[duplicate_item(flow_variant; [ Pending ]; [ Failed ]; [ Done ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(WebCallback);

#[duplicate_item(flow_variant; [ Pending ]; [ Failed ]; [ Done ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn into_inner(self) -> WebCallback { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a WebCallback { &self.0 }
  pub fn site(&self) -> &Site { &self.0.state }
  pub fn kind(&self) -> &WebCallbackKind { &self.0.kind() }
  pub fn resource_id(&self) -> &i32 { self.0.resource_id() }
  pub fn org_id(&self) -> &i32 { self.0.org_id() }
}

#[duplicate_item(
  in_state        is_state        variant(i)             state_struct;
  [ in_pending  ] [ is_pending  ] [ Flow::Pending(i)   ] [ Pending   ];
  [ in_done     ] [ is_done     ] [ Flow::Done(i)      ] [ Done      ];
  [ in_failed   ] [ is_failed   ] [ Flow::Failed(i)    ] [ Failed    ];
)]
impl Flow {
  pub fn in_state(&self) -> ConstataResult<state_struct> {
    if let variant([inner]) = self {
      Ok(inner.clone())
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a WebCallback {
    match self {
      Flow::Pending(a) => a.as_inner(),
      Flow::Done(a) => a.as_inner(),
      Flow::Failed(a) => a.as_inner(),
    }
  }
}

impl WebCallbackHub {
  pub async fn attempt_all_pending(&self) -> ConstataResult<()> {
    let all = self.select()
      .state_eq(WebCallbackState::Pending)
      .next_attempt_on_lte(&Utc::now())
      .all().await?;

    for callback in &all {
      let pending = callback.in_pending()?;
      pending.attempt().await?;
    }

    Ok(())
  }
}

#[derive(Serialize, Deserialize)]
pub struct WebCallbackContent {
  kind: WebCallbackKind,
  resource: WebCallbackResource,
}
#[derive(Serialize, Deserialize)]
enum WebCallbackResource {
  AttestationDone(super::attestation::for_api::Attestation),
}

impl Pending {
  async fn attempt(self) -> ConstataResult<WebCallbackAttempt> {
    let Some(url) = self.as_inner().org().await?.attrs.web_callbacks_url else {
      return self.create_attempt("", WebCallbackResultCode::NoCallbacksUrlForOrg, String::new()).await;
    };

    let result = ureq::post(&url)
      .timeout(std::time::Duration::new(5,0))
      .set("Content-Type", "application/json")
      .send_string(&self.as_inner().request_body().await?);

    match result {
      Ok(response) => {
        let attempt = self.create_attempt(&url, WebCallbackResultCode::Ok, Self::body_of(response)).await?;
        self.into_inner().update().state(WebCallbackState::Done).save().await?;
        return Ok(attempt);
      },
      Err(e) => {
        let (code, text) = match e {
          ureq::Error::Status(code, response) => (
            WebCallbackResultCode::NonSuccessResponse,
            format!("Got {}, {}", code, Self::body_of(response))
          ),
          ureq::Error::Transport(t) => (WebCallbackResultCode::NetworkError, t.to_string()),
        };

        let attempt = self.create_attempt(&url, code, text).await?;
        let inner = self.into_inner();
        let attempts = inner.web_callback_attempt_scope().count().await?;
        let mut update = inner.update().last_attempt_id(Some(*attempt.id()));

        update = if attempts >= 10 {
          update.state(WebCallbackState::Failed)
        } else {
          update.next_attempt_on( Utc::now() + Duration::minutes(5_i64 * 2_i64.pow(attempts as u32 - 1)))
        };
        update.save().await?;

        return Ok(attempt);
      }
    }
  }

  fn body_of(x: ureq::Response) -> String {
    x.into_string().unwrap_or_else(|_| "non_utf8_body".to_string())
  }

  async fn create_attempt(&self, url: &str, result_code: WebCallbackResultCode, mut result_text: String) -> ConstataResult<WebCallbackAttempt> {
    result_text.truncate(5000);
    Ok(self.site().web_callback_attempt().insert(InsertWebCallbackAttempt{
      org_id: *self.org_id(),
      web_callback_id: *self.id(),
      attempted_at: Utc::now(),
      url: url.to_string(),
      result_code,
      result_text,
    }).save().await?)
  }
}

describe! {
  regtest!{ sends_callback_for_attestation_done (site, c, mut chain)
    let alice = c.alice().await;
    let org = alice.org().await
      .update()
      .web_callbacks_url(Some("http://127.0.0.1:1234/callbacks_url".to_string()))
      .save().await?;

    let payloads = vec![
      alice.signed_payload(b"hello world"),
      alice.signed_payload(b"goodbye world"),
    ];
    site.attestation().create(&alice.person().await, &payloads, None, None, None, vec![]).await?;
    let mock = mock_callbacks_url(1, 200);

    assert_eq!(site.web_callback().select().count().await?, 0);
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    assert_eq!(site.web_callback().select().count().await?, 1);
    let mut callback = site.web_callback().select().one().await?;

    let signed: SignedPayload = serde_json::from_str(&callback.request_body().await?)?;
    assert_eq!(signed.signer.to_string(), "msbzXXnZKSyyZpDZh5p9KPPHcAyfuuKycY");
    assert_that!(&String::from_utf8(signed.payload)?, rematch("AttestationDone"));

    assert_that!(&callback.attrs, structure!{ WebCallbackAttrs {
      org_id: eq(*org.id()),
      state: eq(WebCallbackState::Pending),
      kind: eq(WebCallbackKind::AttestationDone),
      resource_id: eq(1),
      last_attempt_id: eq(None),
    }});

    assert_eq!(callback.web_callback_attempt_scope().count().await?, 0);
    for _ in 0..5 {
      site.web_callback().attempt_all_pending().await?;
    }
    callback.reload().await?;
    assert_eq!(callback.web_callback_attempt_scope().count().await?, 1);
    assert_eq!(callback.attrs.state, WebCallbackState::Done);
    mock.assert();
  }

  regtest!{ attempts_several_times_and_fails (site, c, mut chain)
    let alice = c.alice().await;
    alice.org().await.update()
      .web_callbacks_url(Some("http://127.0.0.1:1234/callbacks_url".to_string()))
      .save().await?;

    let payloads = vec![ alice.signed_payload(b"hello world") ];
    site.attestation().create(&alice.person().await, &payloads, None, None, None, vec![]).await?;
    let mock = mock_callbacks_url(10, 500);
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    assert_eq!(site.web_callback().select().count().await?, 1);
    let mut callback = site.web_callback().select().one().await?;

    assert_eq!(*callback.state(), WebCallbackState::Pending);
    assert_eq!(callback.web_callback_attempt_scope().count().await?, 0);
    for _ in 0..5 {
      site.web_callback().attempt_all_pending().await?;
    }
    callback.reload().await?;
    assert_eq!(*callback.state(), WebCallbackState::Pending);
    assert_eq!(callback.web_callback_attempt_scope().count().await?, 1);
    next_attempt_on(&callback, 5);

    let attempt = callback.clone().in_pending()?.attempt().await?;
    assert_eq!(attempt.attrs.result_code, WebCallbackResultCode::NonSuccessResponse);
    callback.reload().await?;
    next_attempt_on(&callback, 10);

    for _ in 0..10 {
      let Ok(pending) = callback.reloaded().await?.in_pending() else { break };
      pending.attempt().await?;
    }

    callback.reload().await?;
    assert_eq!(*callback.state(), WebCallbackState::Failed);
    next_attempt_on(&callback, 1280); // 5 * (2 ** 8)

    mock.assert();
  }

  regtest!{ does_not_create_if_organization_has_no_callbacks_url (site, c, mut chain)
    let alice = c.alice().await;
    let payloads = vec![alice.signed_payload(b"hello world")];
    site.attestation().create(&alice.person().await, &payloads, None, None, None, vec![]).await?;
    assert_eq!(site.web_callback().select().count().await?, 0);
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    assert_eq!(site.web_callback().select().count().await?, 0);
  }

  fn next_attempt_on(callback: &WebCallback, minutes: i64) {
    let delta = *callback.next_attempt_on() - Utc::now();
    assert_that!(&delta, leq(Duration::minutes(minutes)));
    assert_that!(&delta, geq(Duration::minutes(minutes - 1)));
  }
}
