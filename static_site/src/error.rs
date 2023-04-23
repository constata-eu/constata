use std::error::Error as ErrorTrait;
use log::*;
pub use rocket::{
  self,
  get,
  routes,
  serde::json::Json,
  http::ContentType,
  request::Request,
  http::Status,
  response::{self, Responder},
  State
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Rendering(#[from] i18n::tera::Error),
  #[error(transparent)]
  Io(#[from] ex::io::Error),
  #[error(transparent)]
  Renderer(#[from] i18n::renderer::Error),
}

pub type SiteResult<T> = Result<T, Error>;

impl<'r> response::Responder<'r, 'static> for Error {
  fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
    warn!(
      "A wild error appeared: {:?}\n\n{:?}\n",
      &self,
      &self.source()
    );

    let response = ( Status::InternalServerError, "Unexpected Error");

    response.respond_to(request)
  }
}
