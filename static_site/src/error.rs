use std::error::Error as ErrorTrait;
use log::*;
use rocket::{
  self,
  request::Request,
  http::Status,
  response,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] ex::io::Error),
  #[error(transparent)]
  Config(#[from] rocket::figment::Error),
  #[error(transparent)]
  Renderer(#[from] i18n::error::Error),
}

pub type SiteResult<T> = Result<T, Error>;

impl<'r> response::Responder<'r, 'static> for Error {
  fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
    warn!(
      "A wild error appeared: {:?}\n\n{:?}\n",
      &self,
      &self.source()
    );

    let response = match self {
      Error::Renderer(i18n::error::Error::NotFound(n)) => ( Status::NotFound, format!("{n:#?}") ),
      _ => ( Status::InternalServerError, "Unexpected Error".into())
    };

    response.respond_to(request)
  }
}
