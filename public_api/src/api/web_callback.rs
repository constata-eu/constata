use super::*;
use db::*;

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[graphql(description = "A list of WebCallbacks we've sent you or are working on sending you, to help you debug your web callbacks integration.")]
#[serde(rename_all = "camelCase")]
pub struct WebCallback {
  #[graphql(description = "Unique identifier for this Entry, across all Issuances.")]
  pub id: i32,
  #[graphql(description = "The kind of callback, relates to the event that triggered this callback, for example, an attestation being done.")]
  pub kind: WebCallbackKind,
  #[graphql(description = "The related resource ID, has a different meaning depending on the web callback kind. If its about an attestation, the it's the attestation's id.")]
  pub resource_id: i32,
  #[graphql(description = "The state of this callback. Pending, Done or Failed. Callbacks are retried 10 times with exponential backoff. The first attempt is done immediately, the second one 5 minutes later, then at 10 minutes, 20, and so on. All attempts are WebCallbackAttempt.")]
  pub state: WebCallbackState,
  #[graphql(description = "The most recent attempt, if any.")]
  pub last_attempt_id: Option<i32>,
  #[graphql(description = "The date on which this web callback was scheduled. It's around to the time of the event that triggered it, but may be a few seconds later.")]
  pub created_at: UtcDateTime,
  #[graphql(description = "Date in which this WebCallback will be attempted. When the WebCallback is Done or Failed it will remain set to the date of the last attempt.")]
  pub next_attempt_on: UtcDateTime,
  #[graphql(description = "The body of the request we will be sending via POST to your web_callbacks_endpoint")]
  pub request_body: String,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, serde::Serialize, serde::Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct WebCallbackFilter {
  #[arg(long, help="Fetch a specific list of web callbacks by their ids", action=clap::ArgAction::Append)]
  pub ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific web callback by id")]
  pub id_eq: Option<i32>,
  #[arg(long, help="Filter by state")]
  pub state_eq: Option<WebCallbackState>,
  #[arg(long, help="Filter by the associated resource, ie: the attestation id. Use together with kind-eq")]
  pub resource_id_eq: Option<i32>,
  #[arg(long, help="Filter by callback kind.")]
  pub kind_eq: Option<WebCallbackKind>,
}

#[rocket::async_trait]
impl Showable<db::WebCallback, WebCallbackFilter> for WebCallback {
  fn sort_field_to_order_by(field: &str) -> Option<WebCallbackOrderBy> {
    match field {
      "id" => Some(WebCallbackOrderBy::Id),
      "createdAt" => Some(WebCallbackOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<WebCallbackFilter>) -> SelectWebCallback {
    if let Some(f) = filter {
      SelectWebCallback {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        state_eq: f.state_eq,
        resource_id_eq: f.resource_id_eq,
        kind_eq: f.kind_eq,
        ..Default::default()
      }
    } else {
      SelectWebCallback {
        org_id_eq: Some(org_id),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectWebCallback {
    SelectWebCallback { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::WebCallback) -> ConstataResult<Self> {
    let request_body = d.request_body().await?;

    Ok(WebCallback{
      id: d.attrs.id,
      kind: d.attrs.kind,
      resource_id: d.attrs.resource_id,
      state: d.attrs.state,
      last_attempt_id: d.attrs.last_attempt_id,
      created_at: d.attrs.created_at,
      next_attempt_on: d.attrs.next_attempt_on,
      request_body
    })
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "Every attempt we made to deliver a WebCallback to your web_callbacks_url")]
pub struct WebCallbackAttempt {
  #[graphql(description = "Unique identifier for this Entry, across all Issuances.")]
  id: i32,
  #[graphql(description = "The id of the web callback this attempt was made for.")]
  web_callback_id: i32,
  #[graphql(description = "The date in which we made this attempt.")]
  attempted_at: UtcDateTime,
  #[graphql(description = "The url to which we made this attempt, which was your web_callbacks_url at the time.")]
  url: String,
  #[graphql(description = "The result of making this attempt. OK means everything went fine.")]
  result_code: WebCallbackResultCode,
  #[graphql(description = "A text associated to the result code, showing your server's response body or details about network errors.")]
  result_text: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct WebCallbackAttemptFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  web_callback_id_eq: Option<i32>,
  result_code_eq: Option<WebCallbackResultCode>,
}

#[rocket::async_trait]
impl Showable<db::WebCallbackAttempt, WebCallbackAttemptFilter> for WebCallbackAttempt {
  fn sort_field_to_order_by(field: &str) -> Option<WebCallbackAttemptOrderBy> {
    match field {
      "id" => Some(WebCallbackAttemptOrderBy::Id),
      "attemptedAt" => Some(WebCallbackAttemptOrderBy::AttemptedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<WebCallbackAttemptFilter>) -> SelectWebCallbackAttempt {
    if let Some(f) = filter {
      SelectWebCallbackAttempt {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        result_code_eq: f.result_code_eq,
        web_callback_id_eq: f.web_callback_id_eq,
        ..Default::default()
      }
    } else {
      SelectWebCallbackAttempt {
        org_id_eq: Some(org_id),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectWebCallbackAttempt {
    SelectWebCallbackAttempt { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::WebCallbackAttempt) -> ConstataResult<Self> {
    Ok(WebCallbackAttempt{
      id: d.attrs.id,
      web_callback_id: d.attrs.web_callback_id,
      attempted_at: d.attrs.attempted_at,
      url: d.attrs.url,
      result_code: d.attrs.result_code,
      result_text: d.attrs.result_text,
    })
  }
}

constata_lib::describe_one! {
  fulltest!{ can_list_web_callbacks (site, c, client, mut chain)
    use constata_lib::test_support::mock_callbacks_url;

    use gql::{
      *,
      update_web_callbacks_url as update,
      all_web_callbacks as all,
      all_web_callback_attempts as all_attempts,
    };

    let alice = &client.signer;

    let url = "http://127.0.0.1:1234/callbacks_url".to_string();
    let updated: update::ResponseData = client
      .gql( &UpdateWebCallbacksUrl::build_query(update::Variables{ url: Some(url.clone()) }) ).await;
    assert_eq!(updated.update_web_callbacks_url.web_callbacks_url, Some(url));

    {
      let payloads = vec![alice.signed_payload(b"hello world") ];
      site.attestation().create(&alice.person().await, &payloads, None, None, None, vec![]).await?;
      let _mock = mock_callbacks_url(1, 200);
      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      site.web_callback().attempt_all_pending().await?;
    }

    {
      let payloads = vec![alice.signed_payload(b"hello world") ];
      site.attestation().create(&alice.person().await, &payloads, None, None, None, vec![]).await?;
      let _mock = mock_callbacks_url(1, 500);
      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      site.web_callback().attempt_all_pending().await?;
    }

    let search = all::Variables{
      page: Some(0),
      sort_field: Some("createdAt".to_string()),
      per_page: None,
      sort_order: None,
      filter: None,
    };
    let callbacks: all::ResponseData = client.gql(&AllWebCallbacks::build_query(search)).await;
    assert_eq!(callbacks.all_web_callbacks.len(), 2);

    let search_attempts = all_attempts::Variables{
      page: Some(0),
      sort_field: None,
      per_page: None,
      sort_order: None,
      filter: Some(all_attempts::WebCallbackAttemptFilter {
        id_eq: None,
        ids: None,
        web_callback_id_eq: Some(1),
        result_code_eq: None,
      }),
    };
    let attempts: all_attempts::ResponseData = client.gql(&AllWebCallbackAttempts::build_query(search_attempts)).await;
    assert_eq!(attempts.all_web_callback_attempts.len(), 1);
  }
}
