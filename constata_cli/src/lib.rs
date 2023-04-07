/*
 * Allow extracting a specific top level value from a json output to help script writers that don't want jq
 * This lib should re-export all of public_api as 'gql_types'
 * Filters and sort fields should be input types not strings.
 * Refactor "all" queries for Issuances, Entries, Templates, Attestations.
 * API should validate initial entries on issuance creation. Should be a hard error if template does not match.
 *
 * Query variants with and without files.
 *  Lib functions share the same API as the CLI.
 *  If users want a lower level object oriented API we can have a "build your own query" tutorial. <- Not urgent.
 *
 *  - from csv (--csv | --csv-file, --logo-image | --logo-image-file )
 *  - from json (--json | --json-file, --logo-image | --logo-image-file )
 *  - from add-entries (--json | --json-file )
 *  - Account state command should not need an ID, just harcode it in the query.
 *
 *  - assert-issuance-state state --wait [Wait a reasonable time for issuance to be in this state. It's reasonable to expect an Issuance to be created in a few milliseconds, but waits up to 90 minutes for the issuance to be done.]
 *
 *  - See how the generated docs look for constata-client as a library.
 */


use constata_lib::{signed_payload::SignedPayload, models::hasher};
use std::path::PathBuf;
use serde_with::serde_as;
use bitcoin::PublicKey;
use serde::{Deserialize, Serialize};
use simplestcrypt::{deserialize_and_decrypt, DecryptError};
use bitcoin::{ Network, PrivateKey };

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  SignerError(#[from] bitcoin_wallet::error::Error),
  #[error(transparent)]
  ExIO(#[from] ex::io::Error),
  #[error(transparent)]
  StdIO(#[from] std::io::Error),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error(transparent)]
  DerivationError(#[from] bitcoin::util::bip32::Error),
  #[error(transparent)]
  FromUtf8Error(#[from] std::string::FromUtf8Error),
  #[error(transparent)]
  PrivateKey(#[from] bitcoin::util::key::Error),
  #[error("Api error: {0}")]
  ApiError(ApiErrorMessage),
  #[error("Could not decrypt your config file")]
  DecryptError,
  #[error("An error ocurred when encrypting or decrypting the daily key")]
  DailyKeyEncriptionError,
  #[error("The decrypted signing key does not match the expected one")]
  ConfigKeyMismatch,
  #[error("Unexpected error: {0}")]
  Unexpected(String),
  #[error("Could not find {0}")]
  NotFound(String),
}

impl From<base64::DecodeError> for Error {
  fn from(_err: base64::DecodeError) -> Error {
    Error::Unexpected("Could not decode base64 value".to_string())
  }
}

impl From<ureq::Error> for Error {
  fn from(err: ureq::Error) -> Error {
    let message = match err {
      ureq::Error::Transport(transport) => ApiErrorMessage::Transport(transport),
      ureq::Error::Status(code, response) => {
        match response.into_string() {
          Ok(body) => {
            match serde_json::from_str::<GqlResponse<_>>(&body) {
              Ok(errors) => ApiErrorMessage::Gql(errors),
              _ => ApiErrorMessage::Status(code, body),
            }
          },
          Err(_) => ApiErrorMessage::Status(code, "Unexpected non string response from server".into()),
        } 
      },
    };

    Error::ApiError(message)
  }
}

impl From<DecryptError> for Error {
  fn from(_err: DecryptError) -> Error {
    Error::DecryptError
  }
}

pub type ClientResult<T> = std::result::Result<T, Error>;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Config {
  public_key: PublicKey,
  #[serde_as(as = "serde_with::hex::Hex")]
  encrypted_key: Vec<u8>,
  environment: String,
}

pub struct Client {
  key: PrivateKey,
  api_url: &'static str,
  network: Network,
}

impl Client {
  pub fn new(config: &Config, daily_passphrase: &str) -> ClientResult<Client> {
    let (api_url, network) = match config.environment.as_str() {
      "staging" => ("https://api-staging.constata.eu", Network::Bitcoin),
      "production" => ("https://api.constata.eu", Network::Bitcoin),
      _ => ("http://localhost:8000", Network::Regtest),
    };
    let decrypted = deserialize_and_decrypt(daily_passphrase.as_bytes(), &config.encrypted_key)?;
    let key = PrivateKey::from_wif(&String::from_utf8(decrypted)?)?;

    Ok(Client { key, api_url, network })
  }

  pub fn from_config_file(custom_config: Option<PathBuf>, daily_passphrase: &str) -> ClientResult<Client> {
    let config_path = custom_config.unwrap_or_else(|| "constata_conf.json".into() );
    let config: Config = serde_json::from_str(&ex::fs::read_to_string(config_path)?)?;
    Self::new(&config, daily_passphrase)
  }

  pub fn auth_token<'a>(&'a self, nonce: i64, body: &str) -> ClientResult<String> {
    let payload = serde_json::json![{
      "path": "/graphql/",
      "method": "POST",
      "nonce": nonce,
      "body_hash": hasher::hexdigest(body.as_bytes()),
      "query_hash": None::<&str>,
    }].to_string();

    Ok(serde_json::to_string(&self.sign(payload.as_bytes()))?)
  }

  pub fn sign(&self, payload: &[u8]) -> SignedPayload {
    SignedPayload::create(payload, &self.key, self.network) 
  }

  pub fn query<R: for<'a> Deserialize<'a>, V: Serialize>(&self, vars: &V, query: &str) -> ClientResult<R> {
    let body = serde_json::to_string(&serde_json::json![{"query": query, "variables": vars }])?;
    let auth = self.auth_token(chrono::Utc::now().timestamp_millis(), &body)?;
    let endpoint = format!("{}/graphql/", self.api_url);
    let response = ureq::post(&endpoint).set("Authentication", &auth).send_string(&body)?.into_string()?;
    let gql: GqlResponse<R> = serde_json::from_str(&response)
      .map_err(|e| Error::Unexpected(format!("We could not read JSON from response \"{}\" \n\n {}", e, &response)))?;
    match gql {
      GqlResponse{ data: Some(resource), ..} => Ok(resource),
      GqlResponse{ errors, ..} => Err(Error::ApiError(ApiErrorMessage::Gql(GqlResponse{ data: None, errors }))),
    }
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GqlResponse<T> {
  pub data: Option<T>,
  pub errors: Option<Vec<GqlErrorMessage>>,
}

#[derive(Debug)]
pub enum ApiErrorMessage {
  Gql(GqlResponse<serde_json::Value>),
  Status(u16, String),
  Transport(ureq::Transport),
}

impl std::fmt::Display for ApiErrorMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ApiErrorMessage::Gql(x) => {
        write!(f, "GQL, {}", serde_json::to_string_pretty(&x).unwrap_or_else(|_| format!("{:#?}", x) ))?;
      },
      ApiErrorMessage::Status(a,b) => write!(f, "Status, {} {}", a, b)?,
      ApiErrorMessage::Transport(a) => write!(f, "Transport, {:#?}", a)?,
    }
    Ok(())
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GqlErrorMessage {
  pub message: String,
  pub locations: Option<Vec<GqlErrorLocation>>,
  pub path: Option<Vec<String>>,
  pub extensions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GqlErrorLocation {
  pub line: u32,
  pub column: u32,
}

pub mod graphql {
  #[derive(serde::Serialize)]
  #[derive(clap::Args)]
  pub struct Query {
    /// The variables for your query as a JSON string '{"foo": "bar"}'
    #[arg(value_parser=variables_parser)]
    variables: serde_json::Value,

    /// Your full query, check our graphiql explorer to build your own queries.
    query: String,
  }

  fn variables_parser(s: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(&s).map_err(|e| format!("Variables are not valid json: {:#?}", e) )
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<serde_json::Value> {
      client.query(&self.variables, &self.query)
    }
  }
}

pub mod account_state {
  use public_api::controllers::certos::public_graphql::account_state_graphql::AccountState;

  #[derive(serde::Serialize)]
  #[derive(clap::Args)]
  pub struct Query {
    /// Id is ignored for now.
    #[arg(short, long, default_value_t=1)]
    pub id: i32,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<AccountState> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="AccountState")]
        pub inner: AccountState,
      }

      client.query::<Wrapper, Self>(
        self,
        r#"query($id: Int!) {
          AccountState(id: $id) {
            id
            missing
            tokenBalance
            pricePerToken
            maxMonthlyGift
            monthlyGiftRemainder
            parkedCount
            invoices {
              amount
              tokens
              description
              url
              __typename
            }
            pendingTycUrl
            pendingInvoiceLinkUrl
            webCallbacksUrl
            __typename
          }
        }"#
      ).map(|x| x.inner )
    }
  }
}

pub mod create_issuance_from_json {
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::issuance_graphql::{
    Issuance,
    CreateIssuanceFromJsonInput,
  };

  #[derive(serde::Serialize)]
  pub struct Query {
    pub input: CreateIssuanceFromJsonInput,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Issuance> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="createIssuanceFromJson")]
        pub inner: Issuance,
      }

      client.query::<Wrapper, Self>(
        self,
        r#"mutation($input: CreateIssuanceFromJsonInput!) {
          createIssuanceFromJson(input: $input) {
            id
            templateId
            templateName
            templateKind
            state
            name
            createdAt
            errors
            tokensNeeded
            entriesCount
            adminVisitedCount
            publicVisitCount
            __typename
          }
        }"#
      ).map(|x| x.inner )
    }
  }
}

pub mod create_issuance_from_csv {
  use std::path::PathBuf;
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::issuance_graphql::{
    Issuance,
    CreateIssuanceFromCsvInput,
  };

  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[arg(help="The name of the Issuance to be created")]
    pub name: String,

    #[arg(help="csv filename to use as input")]
    pub csv_file_path: PathBuf,

    #[arg(short, long, help="The kind of template to be created if no template_id is given.")]
    pub template_id: Option<i32>,

    #[arg(long, help="The kind of template to be created if no template_id is given.")]
    pub new_kind: Option<TemplateKind>,

    #[arg(long, help="The name of the new template to be created, if no template_id is given.")]
    pub new_name: Option<String>,

    #[arg(long, help="The text to be used as the logo for the new template, if no template_id is given.")]
    pub new_logo_text: Option<String>,

    #[arg(long, help="Path to a PNG or JPEG image to be used as the logo for the new template.\
      If you leave it empty your new_logo_text will be displayed instead."
    )]
    pub new_logo_image: Option<PathBuf>,
  }

  impl Query {
    pub fn run(self, client: &super::Client) -> super::ClientResult<Issuance> {
      #[derive(serde::Serialize)]
      pub struct Vars {
        pub input: CreateIssuanceFromCsvInput,
      }

      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="createIssuanceFromCsv")]
        pub inner: Issuance,
      }

      let new_logo_image = if let Some(x) = &self.new_logo_image {
        Some(base64::encode(ex::fs::read(x)?))
      } else {
        None
      };

      let vars = Vars{
        input: CreateIssuanceFromCsvInput {
          name: self.name,
          template_id: self.template_id,
          new_kind: self.new_kind,
          new_name: self.new_name,
          new_logo_text: self.new_logo_text,
          new_logo_image,
          csv: ex::fs::read_to_string(&self.csv_file_path)?,
        }
      };

      client.query::<Wrapper, Vars>(
        &vars,
        r#"mutation($input: CreateIssuanceFromCsvInput!) {
          createIssuanceFromCsv(input: $input) {
            id
            templateId
            templateName
            templateKind
            state
            name
            createdAt
            errors
            tokensNeeded
            entriesCount
            adminVisitedCount
            publicVisitCount
            __typename
          }
        }"#
      ).map(|x| x.inner )
    }
  }
}

pub mod append_entries_to_issuance {
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::issuance_graphql::{
    Issuance,
    AppendEntriesToIssuanceInput
  };

  #[derive(serde::Serialize)]
  pub struct Query {
    pub input: AppendEntriesToIssuanceInput,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Issuance> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="appendEntriesToIssuance")]
        pub inner: Issuance,
      }

      client.query::<Wrapper, Self>(
        self,
        r#"mutation($input: AppendEntriesToIssuanceInput!) {
          appendEntriesToIssuance(input: $input) {
            id
            templateId
            templateName
            templateKind
            state
            name
            createdAt
            errors
            tokensNeeded
            entriesCount
            adminVisitedCount
            publicVisitCount
            __typename
          }
        }"#
      ).map(|x| x.inner )
    }
  }
}

pub mod all_issuances {
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::{
    ListMetadata,
    issuance_graphql::{
      Issuance,
      IssuanceFilter,
      CreateIssuanceFromCsvInput,
    }
  };

  #[derive(Default, serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[command(flatten)]
    filter: IssuanceFilter,
    #[arg(long,help="The page number to fetch")]
    page: Option<i32>,
    #[arg(long,help="How many pages to fetch")]
    per_page: Option<i32>,
    #[arg(long,help="Field to use for sorting")]
    sort_field: Option<String>,
    #[arg(long,help="Either asc or desc")]
    sort_order: Option<String>,
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize)]
  pub struct Output {
    #[serde(rename="allIssuances")]
    pub all: Vec<Issuance>,
    #[serde(rename="_allIssuancesMeta")]
    pub meta: ListMetadata,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Output> {
      client.query::<Output, Self>(
        self,
        r#"query ($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: IssuanceFilter) {
          allIssuances(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            id
            templateId
            templateName
            templateKind
            state
            name
            createdAt
            errors
            tokensNeeded
            entriesCount
            adminVisitedCount
            publicVisitCount
            __typename
          }
          _allIssuancesMeta(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            count
          }
        }"#
      )
    }
  }
}

pub mod all_entries {
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::{
    ListMetadata,
    entry_graphql::{
      Entry,
      EntryFilter,
    }
  };

  #[derive(Debug, Default, serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[command(flatten)]
    pub filter: EntryFilter,
    #[arg(long,help="The page number to fetch")]
    pub page: Option<i32>,
    #[arg(long,help="How many pages to fetch")]
    pub per_page: Option<i32>,
    #[arg(long,help="Field to use for sorting")]
    pub sort_field: Option<String>,
    #[arg(long,help="Either asc or desc")]
    pub sort_order: Option<String>,
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize)]
  pub struct Output {
    #[serde(rename="allEntries")]
    pub all: Vec<Entry>,
    #[serde(rename="_allEntriesMeta")]
    pub meta: ListMetadata,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Output> {
      client.query::<Output, Self>(
        self,
        r#"query ($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: EntryFilter) {
          allEntries(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            id
            issuanceId
            issuanceName
            rowNumber
            state
            receivedAt
            params
            errors
            documentId
            storyId
            adminVisited
            publicVisitCount
            hasEmailCallback
            emailCallbackSentAt
            downloadProofLinkUrl
            payload
            adminAccessUrl
            __typename
          }
          _allEntriesMeta(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            count
          }
        }"#
      )
    }
  }
}

pub mod all_templates {
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::{
    ListMetadata,
    template_graphql::{
      Template,
      TemplateFilter,
    }
  };

  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[command(flatten)]
    filter: TemplateFilter,
    #[arg(long,help="The page number to fetch")]
    page: Option<i32>,
    #[arg(long,help="How many pages to fetch")]
    per_page: Option<i32>,
    #[arg(long,help="Field to use for sorting")]
    sort_field: Option<String>,
    #[arg(long,help="Either asc or desc")]
    sort_order: Option<String>,
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize)]
  pub struct Output {
    #[serde(rename="allTemplates")]
    pub all: Vec<Template>,
    #[serde(rename="_allTemplatesMeta")]
    pub meta: ListMetadata,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Output> {
      client.query::<Output, Self>(
        self,
        r#"query($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: TemplateFilter) {
          allTemplates(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            id
            name
            kind
            createdAt
            schema {
              name
              optional
              common
              label
              help
              sample
            }
            customMessage
            adminVisitedCount
            entriesCount
            publicVisitCount
            archived
            __typename
          }
          _allTemplatesMeta(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            count
          }
        }"#,
      )
    }
  }
}

pub mod all_attestations {
  pub use constata_lib::models::{self, TemplateKind};
  pub use public_api::controllers::certos::public_graphql::{
    ListMetadata,
    attestation_graphql::{
      Attestation,
      AttestationFilter,
    }
  };

  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[command(flatten)]
    filter: AttestationFilter,
    #[arg(long,help="The page number to fetch")]
    page: Option<i32>,
    #[arg(long,help="How many pages to fetch")]
    per_page: Option<i32>,
    #[arg(long,help="Field to use for sorting")]
    sort_field: Option<String>,
    #[arg(long,help="Either asc or desc")]
    sort_order: Option<String>,
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize)]
  pub struct Output {
    #[serde(rename="allAttestations")]
    pub all: Vec<Attestation>,
    #[serde(rename="_allAttestationsMeta")]
    pub meta: ListMetadata,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Output> {
      client.query::<Output, Self>(
        self,
        r#"query($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: AttestationFilter) {
          allAttestations(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            id
            personId
            orgId
            markers
            openUntil
            state
            parkingReason
            doneDocuments
            parkedDocuments
            processingDocuments
            totalDocuments
            tokensCost
            tokensPaid
            tokensOwed
            buyTokensUrl
            acceptTycUrl
            lastDocDate
            emailAdminAccessUrlTo
            adminAccessUrl
            createdAt
            __typename
          }
          _allAttestationsMeta(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {
            count
          }
        }"#,
      )
    }
  }
}

pub mod is_issuance_created {
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[arg(help="Id of the issuance we want to check is created")]
    issuance_id: i32,
    #[arg(short, long, help="Keep checking and wait this many seconds for issuance to be created.")]
    wait: Option<i32>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<bool> {
      for i in 0..self.wait.map(|x| (x + 1) * 2 ).unwrap_or(1) {
        let result = client.query::<serde_json::Value, Self>(
          &self,
          r#"query Issuance($issuanceId: Int!) {
            Issuance(id: $issuanceId) { state }
          }"#
        )?;

        match result.pointer("/Issuance/state").and_then(|x| x.as_str()) {
          Some("created") => return Ok(true),
          Some("failed") => return Ok(false),
          _ => {},
        }

        if self.wait.map(|x| x * 10 > i ).unwrap_or(false)  {
          std::thread::sleep(std::time::Duration::from_millis(500));
        }
      }
      Ok(false)
    }
  }
}

pub mod is_issuance_done {
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[arg(help="Id of the issuance we want to check is done")]
    id: i32,
    #[arg(short, long, help="Keep checking and wait this many minutes for issuance to be done.")]
    wait: Option<i32>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<bool> {
      for i in 0..self.wait.map(|x| x + 1).unwrap_or(1) {
        let result = client.query::<serde_json::Value, Self>(
          &self,
          r#"query Issuance($id: Int!) {
            Issuance(id: $id) { state }
          }"#
        )?;

        match result.pointer("/Issuance/state").and_then(|x| x.as_str()) {
          Some("done") => return Ok(true),
          Some("failed") => return Ok(false),
          _ => {},
        }

        if self.wait.map(|x| x > i ).unwrap_or(false)  {
          std::thread::sleep(std::time::Duration::from_secs(60));
        }
      }
      Ok(false)
    }
  }
}

pub mod preview {
  use public_api::controllers::certos::public_graphql::entry_graphql::Preview;
  use std::path::PathBuf;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the entry you want preview")]
    pub id: i32,

    #[arg(help="Write the HTML file here, you can then open it with your web browser. \
      Use --json-pointer=html to extract the HTML and print it to stdout.")]
    #[serde(skip)]
    pub out_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Preview> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="Preview")]
        pub inner: Preview,
      }

      let preview = client.query::<Wrapper, Self>(
        self,
        r#"query($id: Int!) {
          Preview(id: $id) {
            id
            html
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(path) = &self.out_file {
        ex::fs::write(path, &preview.html)?;
      }

      Ok(preview)
    }
  }
}

pub mod preview_sample_from_issuance {
  use public_api::controllers::certos::public_graphql::entry_graphql::Preview;
  use std::path::PathBuf;
  use super::*;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the entry you want preview")]
    pub issuance_id: i32,

    #[arg(help="Write the HTML file here, you can then open it with your web browser. \
      Use --json-pointer=html to extract the HTML and print it to stdout.")]
    #[serde(skip)]
    pub out_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(self, client: &super::Client) -> super::ClientResult<Preview> {
      use super::all_entries as e;
      let entries = e::Query{
        filter: e::EntryFilter{ issuance_id_eq: Some(self.issuance_id), ..Default::default() },
        ..Default::default()
      }.run(&client)?;

      let id = entries.all.get(0)
        .ok_or_else(|| Error::NotFound(format!("an entry for issue {}", &self.issuance_id)))?
        .id;

      super::preview::Query{ id: id, out_file: self.out_file }.run(client)
    }
  }
}

pub mod sign_issuance {
  use public_api::controllers::certos::public_graphql::entry_graphql::{SigningIteratorInput, Entry};
  use super::*;

  #[derive(serde::Serialize)]
  pub struct Iter<'a> {
    #[serde(skip)]
    client: &'a Client,
    input: SigningIteratorInput,
    #[serde(skip)]
    current: i32,
    #[serde(skip)]
    total: i32,
  }

  impl<'a> Iter<'a> {
    pub fn new(client: &'a Client, issuance_id: i32) -> ClientResult<Self> {
      use all_entries as e;

      let total = e::Query{
        filter: e::EntryFilter{ issuance_id_eq: Some(issuance_id), ..Default::default() },
        ..Default::default()
      }.run(&client)?.meta.count;

      let current = e::Query{
        filter: e::EntryFilter{
          issuance_id_eq: Some(issuance_id),
          state_eq: Some("signed".to_string()),
          ..Default::default()
        },
        ..Default::default()
      }.run(&client)?.meta.count + 1;

      Ok(Self{
        client,
        input: SigningIteratorInput { issuance_id, entry_id: None, signature: None },
        current,
        total
      })
    }

    pub fn next(&mut self) -> ClientResult<bool> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="signingIterator")]
        pub inner: Option<Entry>,
      }

      let maybe_next = self.client.query::<Wrapper, Self>(
        self,
        r#"mutation ($input: SigningIteratorInput!) {
          signingIterator(input: $input) {
            id
            issuanceId
            issuanceName
            rowNumber
            state
            receivedAt
            params
            errors
            documentId
            storyId
            adminVisited
            publicVisitCount
            hasEmailCallback
            emailCallbackSentAt
            downloadProofLinkUrl
            payload
            adminAccessUrl
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(next) = maybe_next {
        let Some(payload) = &next.payload else { return Ok(false) };
        self.input.entry_id = Some(next.id);
        self.input.signature = Some(self.client.sign(&base64::decode(payload)?).signature.to_base64());
        Ok(true)
      } else {
        Ok(false)
      }
    }

    pub fn sign_all<F: Fn(&Self)>(&mut self, before_each: Option<F>) -> ClientResult<i32> {
      loop {
        if self.next()? {
          before_each.as_ref().map(|f| f(&self) );
        } else {
          break;
        }
        self.current += 1;
      }
      Ok(self.total)
    }
  }

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the issuance whose entries you want to sign")]
    pub id: i32,
    #[arg(short, long, help="Do not output progress information to stdout")]
    pub silent: bool,
  }

  impl Query {
    pub fn run(self, client: &Client) -> ClientResult<i32> {
      let mut iter = Iter::new(client, self.id)?;
      let callback = if self.silent {
        None
      } else {
        Some(|i: &Iter|{ println!("Signing entry {} of {}", i.current, i.total) })
      };
      iter.sign_all(callback)
    }
  }
}

pub mod issuance_export {
  use public_api::controllers::certos::public_graphql::issuance_graphql::IssuanceExport;
  use std::path::PathBuf;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the issuance you want to export as CSV")]
    pub id: i32,

    #[arg(help="Write the CSV file here. \
      Use --json-pointer=csv to extract the CSV and print it to stdout.")]
    #[serde(skip)]
    pub out_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<IssuanceExport> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="IssuanceExport")]
        pub inner: IssuanceExport,
      }

      let preview = client.query::<Wrapper, Self>(
        self,
        r#"query($id: Int!) {
          IssuanceExport(id: $id) {
            id
            csv
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(path) = &self.out_file {
        ex::fs::write(path, &preview.html)?;
      }

      Ok(preview)
    }
  }
}
