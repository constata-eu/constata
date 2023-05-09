mod error;
mod config;
pub use std::path::{Path, PathBuf};
pub use i18n::{Renderer, Lang};
pub use rocket::*;
pub use error::LocalizedResult;
pub use config::Config;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

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

pub async fn rocket() -> Rocket<Build> {
  let cors = rocket_cors::CorsOptions {
    allowed_origins: AllowedOrigins::all(),
    allowed_methods: vec![http::Method::Get].into_iter().map(From::from).collect(),
    allowed_headers: AllowedHeaders::all(),
    allow_credentials: true,
    ..Default::default()
  }
  .to_cors().expect("No pude crear el CORS.");

  build()
    .attach(cors)
    .attach(rocket::fairing::AdHoc::config::<config::Config>())
    .mount("/", routes![ index, public, well_known ])
}
