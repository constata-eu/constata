mod error;
use rocket::*;
use std::path::PathBuf;
use i18n::{make_static_renderer, Lang, LocalizedResponse, Dir};
use error::SiteResult;

make_static_renderer!(RENDERER, "$CARGO_MANIFEST_DIR/src/assets");

#[get("/")]
fn index(lang: Lang) -> SiteResult<LocalizedResponse<'static>> {
  public(lang, PathBuf::from("index.html"))
}

#[get("/<path..>")]
fn public(lang: Lang, path: PathBuf) -> SiteResult<LocalizedResponse<'static>> {
  Ok(RENDERER.render_localized("public", &path, lang, Lang::En)?)
}

#[launch]
async fn rocket() -> Rocket<Build> {
  build().mount("/", routes![ index, public ])
}
