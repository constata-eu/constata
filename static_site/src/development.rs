mod error;
mod config;
use rocket::*;
use std::path::{Path, PathBuf};
use i18n::{Renderer, Lang, LocalizedResponse};
use error::SiteResult;

#[get("/")]
fn index(lang: Lang, config: &rocket::State<config::Config>) -> SiteResult<LocalizedResponse<'static>> {
  public(lang, PathBuf::from("index.html"), config)
}

#[get("/<path..>")]
fn public(lang: Lang, path: PathBuf, config: &rocket::State<config::Config>) -> SiteResult<LocalizedResponse<'static>> {
  Ok(Renderer::new(Path::new("src/assets/"))?.i18n_and_serialize("public", lang, &path, config.inner())?.into_owned())
}

#[launch]
async fn rocket() -> Rocket<Build> {
  build()
    .attach(rocket::fairing::AdHoc::config::<config::Config>())
    .mount("/", routes![ index, public ])
}
