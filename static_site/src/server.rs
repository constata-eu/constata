mod error;
mod config;
pub use std::path::{Path, PathBuf};
pub use i18n::{Renderer, Lang};
pub use rocket::*;
pub use error::LocalizedResult;
pub use config::Config;
use rocket::{
  http::Header,
  fairing::{Fairing, Info, Kind}
};


#[get("/")]
fn index(lang: Lang, config: &State<Config>) -> LocalizedResult {
  public(lang, PathBuf::from("index.html"), config)
}

#[get("/.well-known/<path..>")]
fn well_known(path: PathBuf, config: &State<Config>) -> LocalizedResult {
  public(i18n::Lang::En, Path::new(".well-known/").join(path), config)
}

#[get("/<path..>")]
fn public(lang: Lang, path: PathBuf, config: &State<Config>) -> LocalizedResult {
  super::render(lang, path, config)
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Add CORS headers to responses",
      kind: Kind::Response
    }
  }

  async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
  }
}

pub async fn rocket() -> Rocket<Build> {
  build()
    .attach(CORS)
    .attach(rocket::fairing::AdHoc::config::<config::Config>())
    .mount("/", routes![ index, public, well_known ])
}
