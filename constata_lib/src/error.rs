use sqlx::postgres::PgDatabaseError;
use std::error::Error as ErrorTrait;
use log::*;
use csv;
use rocket::{
  http::Status,
  request::Request,
  response::{self, Responder},
  serde::json::{json, Json},
  warn,
};
use base64;

use juniper::{FieldError, IntoFieldError, ScalarValue, graphql_value};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Ureq(#[from] ureq::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  ZipError(#[from] zip::result::ZipError),
  #[error(transparent)]
  DatabaseError(sqlx::Error),
  #[error("ValidationError on {field}: {message}")]
  Validation { field: String, message: String },
  #[error(transparent)]
  ValidationError(#[from] validator::ValidationErrors),
  #[error(transparent)]
  MailParse(#[from] mailparse::MailParseError),
  #[error("{0}")]
  Stamping(String),
  #[error(transparent)]
  Wallet(#[from] bitcoin_wallet::error::Error),
  #[error(transparent)]
  BitcoinRpc(#[from] bitcoincore_rpc::Error),
  #[error(transparent)]
  Bitcoin(#[from] bitcoin::util::address::Error),
  #[error(transparent)]
  Secp256k1(#[from] bitcoin::secp256k1::Error),
  #[error(transparent)]
  Config(#[from] rocket::figment::Error),
  #[error("Please try later. This action will be ready when Bulletin {0} is published")]
  WaitForBulletin(i32),
  #[error("Document {0} is parked. User needs to accept our Terms and conditions and/or pay before we process it.")]
  DocumentParked(String),
  #[error("You tried to perform an action you're not authorized to perform")]
  Unauthorized,
  #[error(transparent)]
  Utf8Error(#[from] std::str::Utf8Error),
  #[error(transparent)]
  JsonSerde(#[from] serde_json::Error),
  #[error("Unexpected stripe error {0}")]
  Stripe(String),
  #[error("Unexpected stripe error parsing an id {0}")]
  StripeParseId(#[from] stripe::ParseIdError),
  #[error("The action cannot be performed in the required workflow state")]
  InvalidFlowState,
  #[error("Csv Error: {0}")]
  Csv(String),
  #[error(transparent)]
  FromUtf8Error(#[from] std::string::FromUtf8Error),
  #[error(transparent)]
  PrivateKey(#[from] bitcoin::util::key::Error),
  #[error(transparent)]
  MessageSignature(#[from] bitcoin::util::misc::MessageSignatureError),
  #[error("Unexpected legacy failure error {0}")]
  Failure(String),
  #[error("Tera Error: {0}")]
  Tera(String),
  #[error(transparent)]
  S3Creds(#[from] s3::creds::error::CredentialsError),
  #[error(transparent)]
  S3(#[from] s3::error::S3Error),
  #[error("Third party error {0}, {1}")]
  ThirdParty(String, String),
  #[error("Init error {0}")]
  Init(String),
  #[error("Internal unexpected error {0}")]
  Internal(String),
  #[error(transparent)]
  Base64(#[from] base64::DecodeError),
}

impl From<stripe::Error> for Error {
  fn from(err: stripe::Error) -> Error {
    Error::Stripe(format!("{}", err))
  }
}

impl From<failure::Error> for Error {
  fn from(err: failure::Error) -> Error {
    Error::Failure(format!("{}", err))
  }
}

impl From<i18n::tera::Error> for Error {
  fn from(err: i18n::tera::Error) -> Error {
    if let Some(source) = err.source() {
      return Error::Tera(source.to_string());
    }
    Error::Tera(err.to_string())
  }
}

impl From<csv::Error> for Error {
  fn from(err: csv::Error) -> Error {
     let error = match err.kind() {
      csv::ErrorKind::UnequalLengths{..} => "unequal_lengths",
      csv::ErrorKind::Utf8 {..} => "utf8",
      _ => "unexpected",
     };
    Error::validation("payload", error)
  }
}

impl From<sqlx::Error> for Error {
  fn from(err: sqlx::Error) -> Error {
    match err {
      sqlx::Error::Database(ref inner_error) => {
        let pg_error = inner_error.downcast_ref::<PgDatabaseError>();
        error!("PG_ERROR: {:?}", pg_error);
        match pg_error.code() {
          "23505" => {
            Error::validation( "uniqueness", pg_error.detail().unwrap_or("id already exists"))
          },
          "23503" => Error::validation("nonexistent", "references a nonexistent resource"),
          _ => Error::DatabaseError(err),
        }
      }
      _ => Error::DatabaseError(err),
    }
  }
}

impl Error {
  pub fn validation(field: &str, message: &str) -> Error {
    Error::Validation {
      field: field.to_string(),
      message: message.to_string(),
    }
  }

  pub fn third_party(service: &str, message: &str) -> Error {
    Error::ThirdParty(service.to_string(), message.to_string())
  }
}

impl<'r> Responder<'r, 'static> for Error {
  fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
    use bitcoincore_rpc::{
      jsonrpc::error::{Error as JRpcError, RpcError},
      Error as BtcRpcError,
    };

    let response = match self {
      Error::ValidationError(_) => (
        Status::UnprocessableEntity,
        Json(json![{"error": self.to_string()}]),
      ),
      Error::Validation { field, message } => (
        Status::UnprocessableEntity,
        Json(json![{"error": { "field": field, "message": message}}]),
      ),
      Error::WaitForBulletin(_) => (Status::NotFound, Json(json![{"error": self.to_string()}])),
      Error::DatabaseError(sqlx::Error::RowNotFound) => {
        (Status::NotFound, Json(json![{ "error": "Not found" }]))
      }
      Error::BitcoinRpc(BtcRpcError::JsonRpc(JRpcError::Rpc(RpcError { code: -5, .. }))) => {
        (Status::NotFound, Json(json![{ "error": "Not found" }]))
      }
      _ => {
        warn!(
          "A wild error appeared: {:?}\n\n{:?}\n",
          &self,
          &self.source()
        );
        (
          Status::InternalServerError,
          Json(json![{ "error": "Unexpected Error" }]),
        )
      }
    };

    response.respond_to(request)
  }
}

impl<S: ScalarValue> IntoFieldError<S> for Error {
  fn into_field_error(self) -> FieldError<S> {
    match &self {
      Error::ValidationError(_) | Error::WaitForBulletin(_) => FieldError::new(
        self.to_string(),
        graphql_value!(None)
      ),
      Error::Validation { field, message } => FieldError::new(
        self.to_string(),
        graphql_value!({ "error": { "field": field.as_str(), "message": message.as_str()} }),
      ),
      _ => {
        warn!(
          "A wild error appeared: {:?}\n\n{:?}\n",
          &self,
          &self.source()
        );
        FieldError::new( "unexpected error", graphql_value!(None))
      }
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;
