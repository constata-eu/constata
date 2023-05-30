use std::{path::{PathBuf}};
use rocket::get;
use rocket::http::{ContentType, RawStr};
use rocket::response::Redirect;
use include_dir::{include_dir, Dir};

static FILES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static/certos");

#[get("/")]
pub fn app() -> (ContentType, &'static str) {
  (
    ContentType::HTML,
    FILES_DIR
      .get_file("index.html").expect("index.html to be bundled")
      .contents_utf8().expect("Bundled HTML to be utf8")
  )
}

#[get("/vid_chain/redirect_uri?<code>&<scope>&<state>")]
pub async fn vid_chain_redirect_uri(code: &str, scope: &str, state: &str) -> Redirect {
  let uri = format!(
    "/#/vid_chain/redirect_uri?code={code}&scope={scope}&state={state}",
    scope = scope.replace(" ", "+"),
    state = state.replace(" ", "+"),
  );
  Redirect::permanent(uri)
}

#[get("/<file..>", rank=2)]
pub fn build_dir(file: PathBuf) -> Option<(ContentType, &'static [u8])> {
  let path = file.as_path();

  let mime = match path.extension()?.to_str()? {
    "wasm" => ContentType::WASM,
    "ttf"  => ContentType::TTF,
    "png"  => ContentType::PNG,
    "js"   => ContentType::JavaScript,
    "css"  => ContentType::CSS,
    _ => return None,
  };

  Some((mime, FILES_DIR.get_file(path)?.contents()))
}

#[get("/")]
pub fn workroom_redirect() -> rocket::response::Redirect {
  rocket::response::Redirect::to("/")
}

