/// This library has one module for each GraphQL query or mutation available in our API.
/// Each module has its own struct representing the query parameters, and in some cases local parameters as well.
/// We wanted to make it intuitive to transition from the command line subcommands into the API.
/// All subcommands in the command line utility are straightforward representations of queries in this library.
/// For lower level, or more idiomatic access, all types are public and the Client supports arbitrary graphql queries.

/*
 * - Filters, States, and Sort fields should be input types not strings (or they should suggest values in the API at least)
 * - Refactor "all" queries for Issuances, Entries, Templates, Attestations.
 * - SigningIteratorInput should base64 encode when serializing signature automatically.
 * - We're missing all the tests.
 * - assert-issuance-state state --wait [Wait a reasonable time for issuance to be in this state. It's reasonable to expect an Issuance to be created in a few milliseconds, but waits up to 90 minutes for the issuance to be done.]
 *  - See how the generated docs look for constata-client as a library.
 */

use constata_lib::{signed_payload::SignedPayload, models::hasher};
use std::path::PathBuf;
use serde_with::serde_as;
use bitcoin::PublicKey;
use serde::{Deserialize, Serialize};
use simplestcrypt::{deserialize_and_decrypt, DecryptError};
use bitcoin::{ Network, PrivateKey };
pub use public_api::controllers::certos::public_graphql as gql_types;

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
  #[error("You referenced a missing thing: {0}")]
  NotFound(String),
  #[error("You gave an invalid input: {0}")]
  InvalidInput(String),
}

#[macro_export]
macro_rules! check {
  ($exp:expr, $err:ident($($args:tt),*)) => (
    $exp.map_err(|e|{ error!(e, $err($($args),*)) })?
  )
}

#[macro_export]
macro_rules! error {
  ($e:ident($($args:expr),*)) => (
    Error::$e(format!($($args),*))
  );
  ($base_err:ident, $e:ident($($args:expr),*)) => (
    Error::$e(format!("{}, context: {:#?}", format!($($args),*), $base_err))
  );
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
    let config: Config = check!(
      serde_json::from_str(&ex::fs::read_to_string(config_path)?),
      InvalidInput("Your config file is not a valid json")
    );
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
      GqlResponse{ errors: errors @ Some(_), ..} => {
        Err(Error::ApiError(ApiErrorMessage::Gql(GqlResponse{ data: None, errors })))
      },
      GqlResponse{ data: Some(resource), ..} => Ok(resource),
      _ => Err(error!(Unexpected("Server replied with no data and no errors.")))
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
  use super::gql_types::account_state_graphql::AccountState;

  #[derive(serde::Serialize)]
  #[derive(clap::Args)]
  pub struct Query { }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<AccountState> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="AccountState")]
        pub inner: AccountState,
      }

      client.query::<Wrapper, Self>(
        self,
        r#"query{
          AccountState(id: 1) {
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
  pub use super::*;
  pub use constata_lib::models::{self, TemplateKind};
  pub use super::gql_types::issuance_graphql::{
    Issuance,
    CreateIssuanceFromJsonInput,
  };

  #[derive(serde::Serialize)]
  #[derive(clap::Args)]
  pub struct Query {
    #[command(flatten)]
    pub input: CreateIssuanceFromJsonInput,

    #[arg(long, help="A path to a PNG or JPEG file to use as logo in your new template")]
    #[serde(skip)]
    pub new_logo_image_file: Option<PathBuf>,

    #[arg(long, help="A path to file with the JSON for your entries, an alternative to --entry")]
    #[serde(skip)]
    pub json_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(mut self, client: &super::Client) -> super::ClientResult<Issuance> {
      if let Some(ref path) = self.new_logo_image_file {
        self.input.new_logo_image = Some(ex::fs::read(path)?);
      }

      if let Some(ref path) = self.json_file {
        self.input.entries = check!(
          serde_json::from_str(&ex::fs::read_to_string(path)?),
          InvalidInput("your --json-file could not be parsed")
        );
      }

      #[derive(Debug, serde::Deserialize)]
      struct Output {
        #[serde(rename="createIssuanceFromJson")]
        pub inner: Issuance,
      }

      client.query::<Output, Self>(
        &self,
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
  pub use super::gql_types::issuance_graphql::{
    Issuance,
    CreateIssuanceFromCsvInput,
  };

  #[derive(serde::Serialize, clap::Args)]
  pub struct Query {
    #[command(flatten)]
    pub input: CreateIssuanceFromCsvInput,

    #[arg(long, help="A path to a PNG or JPEG file to use as logo in your new template")]
    #[serde(skip)]
    pub new_logo_image_file: Option<PathBuf>,

    #[arg(long, help="A path to a CSV file you want to use for creating this issuance.")]
    #[serde(skip)]
    pub csv_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(mut self, client: &super::Client) -> super::ClientResult<Issuance> {
      if let Some(ref path) = self.new_logo_image_file {
        self.input.new_logo_image = Some(ex::fs::read(path)?);
      }

      if let Some(ref path) = self.csv_file {
        self.input.csv = ex::fs::read_to_string(path)?;
      }

      #[derive(Debug, serde::Deserialize)]
      struct Output {
        #[serde(rename="createIssuanceFromCsv")]
        pub inner: Issuance,
      }

      client.query::<Output, Self>(
        &self,
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
  use super::*;
  pub use super::gql_types::issuance_graphql::{
    Issuance,
    AppendEntriesToIssuanceInput
  };

  #[derive(serde::Serialize, clap::Args)]
  pub struct Query {
    #[command(flatten)]
    pub input: AppendEntriesToIssuanceInput,

    #[arg(long, help="A path to file with the JSON for your entries, an alternative to --entry")]
    #[serde(skip)]
    pub json_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(mut self, client: &super::Client) -> super::ClientResult<Issuance> {
      if let Some(ref path) = self.json_file {
        self.input.entries = check!(
          serde_json::from_str(&ex::fs::read_to_string(path)?),
          InvalidInput("your --json-file could not be parsed")
        );
      }

      #[derive(Debug, serde::Deserialize)]
      struct Output {
        #[serde(rename="appendEntriesToIssuance")]
        pub inner: Issuance,
      }

      client.query::<Output, Self>(
        &self,
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
  pub use super::gql_types::{
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
  pub use super::gql_types::{
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

pub mod all_entries_html_export {
  use super::*;

  #[derive(Debug, Default, serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  #[group(id="all_entries_html_export_query")]
  pub struct Query {
    #[arg(help="Save the verifiable HTML to the given directory if possible. Will fail if it encounters an entry that has no verifiable html available yet.")]
    pub path: PathBuf,

    #[arg(short, long, help="Do not fail if we encounter an entry with no verifiable HTML, skip it instead.")]
    pub dont_fail_on_missing: bool,

    #[command(flatten)]
    pub all_entries: all_entries::Query,
  }

  impl Query {
    pub fn run<F: Fn(i32, i32, &all_entries::Entry)>(&self, client: &super::Client, before_each_save: F) -> super::ClientResult<i32> {
      if !self.path.is_dir() {
        return Err(Error::NotFound(format!("a directory called {}", &self.path.display())))
      }

      let output = self.all_entries.run(client)?;
      let total = output.meta.count;
      let mut current = 1;
      let mut saved = 0;

      for entry in &output.all {
        before_each_save(current, total, entry);

        let exported = super::entry_html_export::Query{
          id: entry.id, 
          out_file: Some(self.path.join(format!("entry_{}.html", entry.id))),
        }.run(client);
        current += 1;

        match exported {
          Ok(_) => saved += 1,
          Err(e) => if !self.dont_fail_on_missing { return Err(e) }
        }
      }

      Ok(saved)
    }
  }
}

pub mod all_templates {
  pub use constata_lib::models::{self, TemplateKind};
  pub use super::gql_types::{
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
  pub use super::gql_types::{
    ListMetadata,
    attestation_graphql::{
      Attestation,
      AttestationFilter,
    }
  };

  #[derive(Debug, serde::Serialize, clap::Args)]
  #[serde(rename_all = "camelCase")]
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

pub mod preview_entry {
  use super::gql_types::entry_graphql::PreviewEntry;
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
    pub fn run(&self, client: &super::Client) -> super::ClientResult<PreviewEntry> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="PreviewEntry")]
        pub inner: PreviewEntry,
      }

      let preview = client.query::<Wrapper, Self>(
        self,
        r#"query($id: Int!) {
          PreviewEntry(id: $id) {
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
  use super::gql_types::entry_graphql::PreviewEntry;
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
    pub fn run(self, client: &super::Client) -> super::ClientResult<PreviewEntry> {
      use super::all_entries as e;
      let entries = e::Query{
        filter: e::EntryFilter{ issuance_id_eq: Some(self.issuance_id), ..Default::default() },
        ..Default::default()
      }.run(&client)?;

      let id = entries.all.get(0)
        .ok_or_else(|| Error::NotFound(format!("an entry for issue {}", &self.issuance_id)))?
        .id;

      super::preview_entry::Query{ id: id, out_file: self.out_file }.run(client)
    }
  }
}

pub mod sign_issuance {
  use super::gql_types::entry_graphql::{SigningIteratorInput, UnsignedEntryPayload};
  use super::*;

  #[derive(serde::Serialize)]
  pub struct Iter<'a> {
    pub input: SigningIteratorInput,
    #[serde(skip)]
    pub client: &'a Client,
    #[serde(skip)]
    pub current: i32,
    #[serde(skip)]
    pub total: i32,
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
      struct Output {
        #[serde(rename="signingIterator")]
        pub inner: Option<UnsignedEntryPayload>,
      }

      let maybe_next = self.client.query::<Output, Self>(
        self,
        r#"mutation ($input: SigningIteratorInput!) {
          signingIterator(input: $input) {
            id
            entry {
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
              payload
              adminAccessUrl
              __typename
            }
            bytes
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(next) = maybe_next {
        self.input.entry_id = Some(next.id);
        self.input.signature = Some(self.client.sign(&next.bytes).signature.to_base64());
        Ok(true)
      } else {
        Ok(false)
      }
    }

    pub fn sign_all<F: Fn(&Self)>(&mut self, before_each: F) -> ClientResult<i32> {
      loop {
        if self.next()? {
          before_each(&self);
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
    #[serde(skip)]
    pub silent: bool,
  }

  impl Query {
    pub fn run<F: Fn(&Iter)>(self, client: &Client, before_each: F) -> ClientResult<i32> {
      Iter::new(client, self.id)?.sign_all(before_each)
    }
  }
}

pub mod issuance_export {
  use super::gql_types::issuance_graphql::IssuanceExport;
  use std::path::PathBuf;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the issuance you want to export as CSV")]
    pub id: i32,

    #[arg(help="Write the CSV file here. \
      Use --json-pointer=csv to extract the CSV and print it to stdout instead")]
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

      let export = client.query::<Wrapper, Self>(
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
        ex::fs::write(path, &export.csv)?;
      }

      Ok(export)
    }
  }
}

pub mod create_attestation {
  use super::*;
  pub use constata_lib::models::{self, TemplateKind};
  pub use super::gql_types::{
    ListMetadata,
    attestation_graphql::{
      Attestation,
      AttestationInput,
      AttestationFilter,
    }
  };

  #[derive(serde::Serialize, serde::Deserialize)]
  pub struct QueryWithAttestationInput {
    input: AttestationInput,
  }

  impl From<AttestationInput> for QueryWithAttestationInput {
    fn from(input: AttestationInput) -> Self {
      QueryWithAttestationInput{ input }
    }
  }

  impl QueryWithAttestationInput {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<Attestation> {
      #[derive(Debug, serde::Deserialize)]
      struct Output {
        #[serde(rename="createAttestation")]
        pub inner: Attestation,
      }

      client.query::<Output, Self>(
        self,
        r#"mutation CreateAttestation($input: AttestationInput!) {
          createAttestation(input: $input) {
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
        }"#,
      ).map(|x| x.inner )
    }

  }

  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    /// A list of paths to the files to add to your attestation.
    #[arg(short, long="path", value_name="PATH", action=clap::ArgAction::Append,
      help="Path to a file you want to include in your attestation. You can repeat this argument to add many.")]
    pub paths: Vec<PathBuf>,

    /// An attestation allows appending documents up until a certain date. If you don't chose a date, no appending will be allowed.
    pub open_until: Option<constata_lib::models::UtcDateTime>,

    /// Markers is a text that can be used for searching this attestation later. Markers cannot be updated after creation.
    pub markers: Option<String>,

    /// A list of email addresses to notify when the documents are attested.
    /// Constata will email them an administrative access link to view, download or share the document certificate.
    /// You can pass an empty list if you want to omit Constata's emails, and manage distribution of the attestation on your own.
    #[arg(short, long, value_name="PATH", action=clap::ArgAction::Append,
      help="Email an admin access link to view, download and share this attestation to the given addresses.")]
    pub email_admin_access_url_to: Vec<String>,
  }

  impl Query {
    pub fn run(self, client: &Client) -> ClientResult<Attestation> {
      let documents: Vec<SignedPayload> = self.paths.iter()
        .map(|f| ex::fs::read(f).map(|x| client.sign(&x) ).into() )
        .collect::<Result<Vec<_>, ex::io::Error>>()?;

      QueryWithAttestationInput::from(AttestationInput{
        documents,
        open_until: self.open_until,
        markers: self.markers,
        email_admin_access_url_to: self.email_admin_access_url_to,
      }).run(client)
    }
  }
}

pub mod attestation_html_export {
  use std::path::PathBuf;
  use super::gql_types::attestation_graphql::AttestationHtmlExport;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the attestation your want to export.")]
    pub id: i32,

    #[arg(help="Write the HTML file here, you can then open it with your web browser. \
      Use --json-pointer=html to extract the HTML and print it to stdout.")]
    #[serde(skip)]
    pub out_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<AttestationHtmlExport> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="AttestationHtmlExport")]
        pub inner: AttestationHtmlExport,
      }

      let export = client.query::<Wrapper, Self>(
        self,
        r#"query AttestationHtmlExport($id: Int!) {
          AttestationHtmlExport(id: $id) {
            id
            attestation {
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
            verifiableHtml
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(path) = &self.out_file {
        ex::fs::write(path, &export.verifiable_html)?;
      }

      Ok(export)
    }
  }
}

pub mod attestation_state {
  use serde::{Serialize, Deserialize};

  #[derive(Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  pub struct Query {
    #[arg(help="Id of the attestation we want to check")]
    id: i32,
    #[arg(help="The state we expect the attestation to be in")]
    state: State,
    #[arg(short, long, help="Keep checking and wait this many seconds for issuance to be created.")]
    wait: Option<i32>,
  }

  #[derive(Debug, Clone, Deserialize, PartialEq, Serialize, clap::ValueEnum)]
  #[serde(rename_all = "UPPERCASE")]
  pub enum State {
    Parked,
    Processing,
    UpdatesParked,
    UpdatesProcessing,
    Done,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<bool> {
      for i in 0..self.wait.map(|x| (x + 1) * 2 ).unwrap_or(1) {
        let result = client.query::<serde_json::Value, Self>(
          &self,
          r#"query Attestation($id: Int!) {
            Attestation(id: $id) { state }
          }"#
        )?;

        let expected = serde_json::to_string(&self.state)?;

        if matches!(result.pointer("/Attestation/state").and_then(|x| x.as_str()),  Some(e) if e == expected) {
          return Ok(true);
        }

        if self.wait.map(|x| x * 10 > i ).unwrap_or(false)  {
          std::thread::sleep(std::time::Duration::from_millis(500));
        }
      }
      Ok(false)
    }
  }
}

pub mod entry_html_export {
  use std::path::PathBuf;
  use super::gql_types::entry_graphql::EntryHtmlExport;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the entry your want to export.")]
    pub id: i32,

    #[arg(help="Write the verifiable HTML file here, you can then open it with your web browser. \
      Use --json-pointer=html to extract the HTML and print it to stdout.")]
    #[serde(skip)]
    pub out_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<EntryHtmlExport> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="EntryHtmlExport")]
        pub inner: EntryHtmlExport,
      }

      let export = client.query::<Wrapper, Self>(
        self,
        r#"query EntryHtmlExport($id: Int!) {
          EntryHtmlExport(id: $id) {
            id
            entry {
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
              payload
              adminAccessUrl
              __typename
            }
            verifiableHtml
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(path) = &self.out_file {
        ex::fs::write(path, &export.verifiable_html)?;
      }

      Ok(export)
    }
  }
}

pub mod unsigned_entry_payload {
  use std::path::PathBuf;
  use super::gql_types::entry_graphql::UnsignedEntryPayload;

  #[derive(clap::Args)]
  #[derive(serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Query {
    #[arg(help="id of the entry your want to export.")]
    pub id: i32,

    #[arg(help="Write the verifiable HTML file here, you can then open it with your web browser. \
      Use --json-pointer=html to extract the HTML and print it to stdout.")]
    #[serde(skip)]
    pub out_file: Option<PathBuf>,
  }

  impl Query {
    pub fn run(&self, client: &super::Client) -> super::ClientResult<UnsignedEntryPayload> {
      #[derive(Debug, serde::Deserialize)]
      struct Wrapper {
        #[serde(rename="UnsignedEntryPayload")]
        pub inner: UnsignedEntryPayload,
      }

      let export = client.query::<Wrapper, Self>(
        self,
        r#"query UnsignedEntryPayload($id: Int!) {
          UnsignedEntryPayload(id: $id) {
            id
            entry {
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
              payload
              adminAccessUrl
              __typename
            }
            bytes
            __typename
          }
        }"#
      ).map(|x| x.inner )?;

      if let Some(path) = &self.out_file {
        ex::fs::write(path, &export.bytes)?;
      }

      Ok(export)
    }
  }
}

pub mod all_attestations_html_export {
  use super::*;

  #[derive(Debug, serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  #[derive(clap::Args)]
  #[group(id="all_attestations_html_export_query")]
  pub struct Query {
    #[arg(help="Save the verifiable HTMLs to the given directory if possible. Will fail if it encounters an attestation that has no verifiable html available yet.")]
    pub path: PathBuf,

    #[arg(short, long, help="Do not fail if we encounter an attestation with no verifiable HTML, skip it instead.")]
    pub dont_fail_on_missing: bool,

    #[command(flatten)]
    pub all_attestations: all_attestations::Query,
  }

  impl Query {
    pub fn run<F: Fn(i32, i32, &all_attestations::Attestation)>(&self, client: &super::Client, before_each_save: F) -> super::ClientResult<i32> {
      if !self.path.is_dir() {
        return Err(Error::NotFound(format!("a directory called {}", &self.path.display())))
      }

      let output = self.all_attestations.run(client)?;
      let total = output.meta.count;
      let mut current = 1;
      let mut saved = 0;

      for entry in &output.all {
        before_each_save(current, total, entry);

        let exported = attestation_html_export::Query{
          id: entry.id, 
          out_file: Some(self.path.join(format!("attestation_{}.html", entry.id))),
        }.run(client);
        current += 1;

        match exported {
          Ok(_) => saved += 1,
          Err(e) => if !self.dont_fail_on_missing { return Err(e) }
        }
      }

      Ok(saved)
    }
  }
}
