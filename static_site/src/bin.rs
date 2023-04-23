use std::{path::{PathBuf}};
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
use i18n::{make_static_renderer, Renderer, Lang, HtmlWithLocale, LocalizedResponse, Context, Dir};

mod error;
pub use error::{Error, SiteResult};

make_static_renderer!(static ref RENDERER: Renderer<&'static Dir<'static>>, "$CARGO_MANIFEST_DIR/src/assets");

#[get("/")]
pub fn index(lang: Lang) -> SiteResult<LocalizedResponse> {
  public(lang, PathBuf::from("index.html"))
}

#[get("/<path..>")]
pub fn public(lang: Lang, path: PathBuf) -> SiteResult<LocalizedResponse> {
  Ok(RENDERER.render_localized("public", &path, lang, Lang::En)?)
}

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  rocket::build().mount("/", routes![ index, public ])
}
