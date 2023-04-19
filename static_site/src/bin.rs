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
use i18n::{make_static_renderer, Renderer, Lang, HtmlWithLocale};

mod error;
pub use error::{Error, SiteResult};

make_static_renderer!(static ref RENDERER: Renderer, "$CARGO_MANIFEST_DIR/templates");
static ASSETS: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets");

#[get("/<page..>")]
pub fn page(lang: Lang, page: PathBuf) -> SiteResult<HtmlWithLocale> {
  let template = if page.to_str().map(|s| s.len() == 0).unwrap_or(true) {
    format!("pages/index.html")
  } else {
    format!("pages/{}", page.display())
  };

  Ok(HtmlWithLocale::no_context(&RENDERER, lang, &template));
}

#[get("/styles/<style..>")]
pub fn style(style: PathBuf) -> SiteResult<String> {
  Ok(RENDERER.static_file(&format!("styles/{}", style.display()))?)
}

#[get("/assets/<file..>")]
pub fn asset(file: PathBuf) -> Option<(ContentType, &'static [u8])> {
  let path = file.as_path();

  let mime = match path.extension()?.to_str()? {
    "wasm" => ContentType::WASM,
    "ttf"  => ContentType::TTF,
    "png"  => ContentType::PNG,
    "js"   => ContentType::JavaScript,
    "css"  => ContentType::CSS,
    _ => return None,
  };

  Some((mime, ASSETS.get_file(path)?.contents()))
}

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  rocket::build().mount("/", routes![ style, asset, page ])
}

