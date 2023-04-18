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

mod error;
pub use error::{Error, SiteResult};

#[get("/<page..>")]
pub fn page(page: PathBuf) -> SiteResult<(ContentType, String)> {
  let template = if page.to_str().map(|s| s.len() == 0).unwrap_or(true) {
    format!("pages/index.html")
  } else {
    format!("pages/{}", page.display())
  };

  Ok(( ContentType::HTML, make_tera().render(&template, &Context::new())? ))
}

#[get("/style/<style..>")]
pub fn style(style: PathBuf) -> SiteResult<(ContentType, String)> {
  Ok((
    ContentType::HTML,
    make_tera().render(&format!("styles/{}", style.display()), &Context::new())?
  ))
}

#[get("/static/<file..>")]
pub fn asset(file: PathBuf) -> Option<(ContentType, Vec<u8>)> {
  let path = file.as_path();

  let mime = match path.extension()?.to_str()? {
    "wasm" => ContentType::WASM,
    "ttf"  => ContentType::TTF,
    "png"  => ContentType::PNG,
    "js"   => ContentType::JavaScript,
    "css"  => ContentType::CSS,
    _ => return None,
  };

  Some((mime, ex::fs::read(format!("assets/{}", file.display())).ok()?))
}

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  rocket::build().mount("/", routes![ style, asset, page ])
}

pub fn make_tera() -> Tera {
  use glob::glob;

  let mut templates = Tera::default();

  let mut entries: Vec<PathBuf> = glob("templates/**/[!.]*")
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
      .strip_prefix("templates")
      .expect("template/ prefix could not be stripped")
      .display()
      .to_string();

    let contents = ex::fs::read_to_string(&entry).unwrap();

    if entry.ends_with(".scss") {
      let style = grass::from_string(
        contents,
        &grass::Options::default(),
      ).expect(&format!("Failed to compile SCSS: {}", pathname));
      templates.add_raw_template(&pathname, &style).expect("could not add template");
    } else {
      templates.add_raw_template(&pathname, &contents).expect("could not add template");
    }
  }
  println!("Templates {:#?}", templates.get_template_names().map(|x| x.to_string() ).collect::<Vec<String>>());
  
  templates
}
