/*
 *
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
  #[error("Unexpected error {0}")]
  Unexpected(&'static str),
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
}

impl Client {
  pub fn new(config: &Config, daily_passphrase: &str) -> ClientResult<Client> {
    let api_url = match config.environment.as_str() {
      "staging" => "https://api-staging.constata.eu",
      "production" => "https://api.constata.eu",
      _ => "http://localhost:8000",
    };
    let decrypted = deserialize_and_decrypt(daily_passphrase.as_bytes(), &config.encrypted_key)?;
    let key = PrivateKey::from_wif(&String::from_utf8(decrypted)?)?;

    Ok(Client { key, api_url })
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

    Ok(serde_json::to_string(&SignedPayload::create(payload.as_bytes(), &self.key, Network::Regtest))?)
  }

  pub fn query<R: for<'a> Deserialize<'a>, V: Serialize>(&self, vars: &V, query: &str) -> ClientResult<R> {
    let body = serde_json::to_string(&serde_json::json![{"query": query, "variables": vars }])?;
    let auth = self.auth_token(chrono::Utc::now().timestamp_millis(), &body)?;
    let endpoint = format!("{}/graphql/", self.api_url);
    let response: GqlResponse<R> = ureq::post(&endpoint).set("Authentication", &auth).send_string(&body)?.into_json()?;
    match response {
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
        write!(f, "GQL, {}", serde_json::to_string_pretty(&x).unwrap_or_else(|_| format!("{:?}", x) ))?;
      },
      ApiErrorMessage::Status(a,b) => write!(f, "Status, {} {}", a, b)?,
      ApiErrorMessage::Transport(a) => write!(f, "Transport, {:?}", a)?,
    }
    Ok(())
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GqlErrorMessage {
  pub message: String,
  pub locations: Option<Vec<GqlErrorLocation>>,
  pub path: Option<Vec<String>>,
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
    serde_json::from_str(&s).map_err(|e| format!("Variables are not valid json: {:?}", e) )
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
        r#"mutation CreateIssuanceFromJson($input: CreateIssuanceFromJsonInput!) {
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
        r#"mutation CreateIssuanceFromCsv($input: CreateIssuanceFromCsvInput!) {
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
        r#"mutation CreateIssuanceFromCsv($input: CreateIssuanceFromCsvInput!) {
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
