use super::*;
pub use constata_lib::models::{self, SelectVcRequest, VcRequestOrderBy, VcRequestState};

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "A verifiable credentials presentation request.")]
pub struct VcRequest {
  #[graphql(description = "Unique numeric identifier of this request.")]
  id: i32,
  #[graphql(description = "The VC Prompt which created this presentation request.")]
  prompt_id: i32,
  #[graphql(description = "The presentation request state")]
  state: VcRequestState,
  #[graphql(description = "A snake_cased_string with further information about the current state. Can be translated in the front end.")]
  state_notes: Option<String>,
  #[graphql(description = "The full VC presentation received from the end user responding to the request.")]
  vc_presentation: Option<String>,
  #[graphql(description = "DID of the user responding to this request.")]
  did: Option<String>,
  #[graphql(description = "The date in which this presentation request was made.")]
  started_at: UtcDateTime,
  #[graphql(description = "The time at which thes presentation request reached its final state.")]
  finished_at: Option<UtcDateTime>,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, serde::Serialize, serde::Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct VcRequestFilter {
  #[arg(long, help="Fetch a specific list of requests by their ids", action=clap::ArgAction::Append)]
  ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific request by id")]
  id_eq: Option<i32>,
  #[arg(long, help="Fetch requests in the given state")]
  state_eq: Option<VcRequestState>,
  #[arg(long, help="Fetch requests by the respondent DID")]
  did_like: Option<String>,
  #[arg(long, help="Fetch requests by the respondent DID")]
  finished_at_is_set: Option<bool>,
}

#[rocket::async_trait]
impl Showable<models::VcRequest, VcRequestFilter> for VcRequest {
  fn sort_field_to_order_by(field: &str) -> Option<VcRequestOrderBy> {
    match field {
      "id" => Some(VcRequestOrderBy::Id),
      "startedAt" => Some(VcRequestOrderBy::StartedAt),
      "finishedAt" => Some(VcRequestOrderBy::FinishedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<VcRequestFilter>) -> SelectVcRequest {
    if let Some(f) = filter {
      SelectVcRequest {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        state_eq: f.state_eq,
        finished_at_is_set: f.finished_at_is_set,
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectVcRequest {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectVcRequest {
    SelectVcRequest { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: models::VcRequest, _with_payload: bool) -> MyResult<Self> {
    Ok(VcRequest {
      id: d.attrs.id,
      prompt_id: d.attrs.vc_prompt_id,
      state: d.attrs.state,
      state_notes: d.attrs.state_notes,
      vc_presentation: d.attrs.vc_presentation,
      did: d.attrs.did,
      started_at: d.attrs.started_at,
      finished_at: d.attrs.finished_at,
    })
  }
}

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "A verifiable credentials presentation request.")]
pub struct KioskVcRequest {
  #[graphql(description = "Unique numeric identifier of this request.")]
  id: i32,
  #[graphql(description = "Public description.")]
  description: String,
  #[graphql(description = "Url for VidWallet users. They must open it with their QR code scanner.")]
  vidchain_url: String,
  #[graphql(description = "The presentation request state")]
  state: VcRequestState,
  #[graphql(description = "A snake_cased_string with further information about the current state. Can be translated in the front end.")]
  state_notes: Option<String>,
  #[graphql(description = "The date in which this presentation request was made.")]
  started_at: UtcDateTime,
}

impl KioskVcRequest {
  pub async fn create( context: &Context ) -> FieldResult<KioskVcRequest> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      let request = context.site.vc_prompt().select()
        .access_token_id_eq(token.attrs.id)
        .one().await?
        .create_request().await?;

      Self::db_to_graphql(request).await
    } else {
      Err(field_error("access", "invalid auth token"))
    }
  }

  pub async fn get(context: &Context, id: i32) -> FieldResult<KioskVcRequest> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      let prompt = context.site.vc_prompt().select()
        .access_token_id_eq(token.attrs.id)
        .one().await?;
      let request = prompt.vc_request_scope().id_eq(id).one().await?;

      Self::db_to_graphql(request).await
    } else {
      Err(field_error("access", "invalid verifiable credentials request token"))
    }
  }

  pub async fn update( context: &Context, code: &str ) -> FieldResult<KioskVcRequest> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      let request = context.site.vc_request().select()
        .access_token_id_eq(token.attrs.id)
        .one().await?
        .resolve_with_vidchain_code(code).await?;

      Self::db_to_graphql(request).await
    } else {
      Err(field_error("access", "invalid auth token"))
    }
  }

  pub async fn db_to_graphql(d: models::VcRequest) -> FieldResult<KioskVcRequest> {
    let description = d.vc_prompt().await?.attrs.name;
    let vidchain_url = d.vidchain_url().await?;

    Ok(KioskVcRequest {
      id: d.attrs.id,
      description,
      vidchain_url,
      state: d.attrs.state,
      state_notes: d.attrs.state_notes,
      started_at: d.attrs.started_at,
    })
  }
}
