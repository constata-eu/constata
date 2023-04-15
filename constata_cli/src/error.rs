use crate::client::*;

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

impl From<simplestcrypt::DecryptError> for Error {
  fn from(_err: simplestcrypt::DecryptError) -> Error {
    Error::DecryptError
  }
}

pub type ClientResult<T> = std::result::Result<T, Error>;
