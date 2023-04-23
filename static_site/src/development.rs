use std::path::PathBuf;
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
use std::path::Path;

use i18n::{make_static_renderer, Renderer, Lang, HtmlWithLocale, LocalizedResponse, Context, Dir};

mod error;
pub use error::{Error, SiteResult};

#[get("/")]
pub fn index(lang: Lang) -> SiteResult<LocalizedResponse> {
  public(lang, PathBuf::from("index.html"))
}

#[get("/<path..>")]
pub fn public(lang: Lang, path: PathBuf) -> SiteResult<LocalizedResponse> {
  Ok(Renderer::new(Path::new("src/assets/"))?.render_localized("public", &path, lang, Lang::En)?)
}

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  rocket::build().mount("/", routes![ index, public ])
}
