mod error;
use rocket::*;
use std::path::{Path, PathBuf};
use i18n::{Renderer, Lang, LocalizedResponse};
use error::SiteResult;

#[get("/")]
fn index(lang: Lang) -> SiteResult<LocalizedResponse<'static>> {
  public(lang, PathBuf::from("index.html"))
}

#[get("/<path..>")]
fn public(lang: Lang, path: PathBuf) -> SiteResult<LocalizedResponse<'static>> {
  Ok(Renderer::new(Path::new("src/assets/"))?.i18n("public", lang, &path)?.into_owned())
}

#[launch]
async fn rocket() -> Rocket<Build> {
  build().mount("/", routes![ index, public ])
}
