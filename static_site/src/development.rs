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
use i18n::{Context, Tera, grass};
use std::path::Path;

mod error;
pub use error::{Error, SiteResult};

#[get("/<path..>", rank=2)]
pub fn public(path: PathBuf) -> SiteResult<Option<(ContentType, String)>> {
  let asset = if path.to_str().map(|s| s.len() == 0).unwrap_or(true) {
    format!("/index.html")
  } else {
    path.display().to_string()
  };

  let Some(ext) = path.extension().and_then(|x| x.to_str() ) else { return Ok(None); };

  let mime = match ext {
    "wasm" => ContentType::WASM,
    "ttf"  => ContentType::TTF,
    "png"  => ContentType::PNG,
    "js"   => ContentType::JavaScript,
    "css"  => ContentType::CSS,
    "scss"  => ContentType::CSS,
    "svg"  => ContentType::SVG,
    "html"  => ContentType::HTML,
    _ => return Ok(None),
  };

  Ok(Some((mime, make_tera().render(&format!("public/{asset}"), &Context::new())? )))
}

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  rocket::build().mount("/", routes![ public ])
}

pub fn make_tera() -> Tera {
  use glob::glob;

  let mut templates = Tera::default();

  let mut entries: Vec<PathBuf> = glob("src/assets/**/[!.]*")
    .expect("Autoreload glob pattern was invalid")
    .map(|result|{
      result
        .expect("Glob result was invalid")
        .to_path_buf()
    }).collect();

  entries.sort_by(|a,b|{
    let a_depth = a.ancestors().count();
    let b_depth = b.ancestors().count();
    let a_priority = a.starts_with("_");
    let b_priority = b.starts_with("_");

    if a_priority == b_priority {
      a_depth.cmp(&b_depth)
    } else {
      b_priority.cmp(&a_priority)
    }
  });

  for entry in &entries {
    if !entry.is_file() {
      continue;
    }
    let pathname = entry
      .strip_prefix("src/assets")
      .expect("src/assets prefix could not be stripped")
      .display()
      .to_string();

    let contents = ex::fs::read_to_string(&entry).unwrap();

    if pathname.ends_with(".css") {
      let style = grass::from_string(
        contents,
        &grass::Options::default().load_path(Path::new("./src/scss/")),
      ).expect(&format!("Failed to compile SCSS: {}", pathname));
      templates.add_raw_template(&pathname, &style).expect("could not add template");
    } else {
      templates.add_raw_template(&pathname, &contents).expect("could not add template");
    }
  }
  println!("Assets {:#?}", templates.get_template_names().map(|x| x.to_string() ).collect::<Vec<String>>());
  
  templates
}
