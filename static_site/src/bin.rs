mod error;
mod config;
use rocket::*;
use std::path::PathBuf;
use i18n::{make_static_renderer, Lang, LocalizedResponse};
use error::SiteResult;

make_static_renderer!(RENDERER, "$CARGO_MANIFEST_DIR/src/assets");

#[get("/")]
fn index(lang: Lang, config: &rocket::State<config::Config>) -> SiteResult<LocalizedResponse<'static>> {
  public(lang, PathBuf::from("index.html"), config)
}

#[get("/<path..>")]
fn public(lang: Lang, path: PathBuf, config: &rocket::State<config::Config>) -> SiteResult<LocalizedResponse<'static>> {
  Ok(RENDERER.i18n_and_serialize("public", lang, &path, config.inner())?)
}

#[launch]
async fn rocket() -> Rocket<Build> {
  build()
    .attach(rocket::fairing::AdHoc::config::<config::Config>())
    .mount("/", routes![ index, public ])
}
