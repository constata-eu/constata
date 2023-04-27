#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("File not found {0}")]
  NotFound(String),
  #[error("Unexpected error {0}")]
  Internal(String),
}

impl From<glob::PatternError> for Error {
  fn from(_err: glob::PatternError) -> Error {
    Error::Internal("Invalid glob pattern".to_string())
  }
}

impl From<std::path::StripPrefixError> for Error {
  fn from(_err: std::path::StripPrefixError) -> Error {
    Error::Internal("could not remove prefix".to_string())
  }
}

impl From<glob::GlobError> for Error {
  fn from(_err: glob::GlobError) -> Error {
    Error::Internal("Invalid glob operation".to_string())
  }
}

impl From<ex::io::Error> for Error {
  fn from(err: ex::io::Error) -> Error {
    Error::NotFound(format!("IO Error: {err:?}"))
  }
}

impl From<Box<grass::Error>> for Error {
  fn from(err: Box<grass::Error>) -> Error {
    Error::Internal(format!("Error rendering stylesheet: {err:?}"))
  }
}

impl From<tera::Error> for Error {
  fn from(err: tera::Error) -> Error {
    match err.kind {
      tera::ErrorKind::TemplateNotFound(e) => Error::NotFound(e),
      _ => Error::Internal(format!("Template error: {err:?}"))
    }
  }
}

pub type RendererResult<T> = Result<T, Error>;
