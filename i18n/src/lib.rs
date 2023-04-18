pub mod renderer;
pub mod translations;
pub use tera::{self, Context, Tera, Result as TeraResult};
pub use grass;
pub use include_dir::{include_dir, Dir, DirEntry};
pub use formatx;
pub use renderer::Renderer;

use rocket::{
  self,
  http::ContentType,
  response,
  request::{self, FromRequest, Outcome, Request},
};

#[derive(sqlx::Type, Copy, Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[sqlx(type_name = "language", rename_all = "lowercase")]
pub enum Lang {
  Es,
  En,
}

impl sqlx::postgres::PgHasArrayType for Lang {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_language")
  }
}

#[macro_export]
macro_rules! t {
  ($lang:expr, $translation:ident) => {
    $lang.translations().$translation.to_string()
  };
  ($lang:expr, $translation:ident, $($e:tt)*) => {
    i18n::formatx::formatx!($lang.translations().$translation, $($e)*).expect("This can only fail if we mismatch translation params ourselves.")
  };
}

impl Lang {
  pub fn code(&self) -> &'static str {
    match self {
      Lang::Es => "es",
      Lang::En => "en",
    }
  }

  pub fn translations(&self) -> translations::TranslatedStrings {
    match self {
      Lang::Es => translations::SPANISH_STRINGS,
      Lang::En => translations::ENGLISH_STRINGS,
    }
  }

  pub fn html(self, template_name: &str, ctx: &Context) -> TeraResult<HtmlWithLocale> {
    Ok(HtmlWithLocale{ lang: self, content: render(self, template_name, ctx)? })
  }

  pub fn html_from_serialize<S: serde::Serialize>(self, template_name: &str, o: &S) -> TeraResult<HtmlWithLocale> {
    Ok(HtmlWithLocale{ lang: self, content: render_from_serialize(self, template_name, o)? })
  }

  pub fn html_bare(self, template_name: &str) -> TeraResult<HtmlWithLocale> {
    Ok(HtmlWithLocale{
      lang: self,
      content: render_from_serialize(self, template_name, &serde_json::json!({}))?,
    })
  }

  pub fn detect_from_text(text: &str) -> Self {
    use whatlang::{Detector, Lang as L};
    if let Some(L::Spa) = Detector::with_allowlist(vec![L::Eng, L::Spa]).detect_lang(text) {
      Self::Es
    } else {
      Self::En
    }
  }

  pub fn find_in_request_headers(r: &Request) -> Option<Lang> {
    r.headers()
      .get_one("Accept-Language")
      .unwrap_or("en")
      .split(",")
      // Get the locale, not the country code
      .filter_map(|l| l.split(|c| c == '-' || c == ';').nth(0))
      // Get the first requested locale we support
      .find(|l| *l == "en" || *l == "es" )
      .map(|l| if l == "es" { Lang::Es } else { Lang::En })
  }
}

static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

fn file_name<'a>(e: &'a DirEntry) -> &'a str {
  e.path().file_name().unwrap().to_str().unwrap()
}

fn path_name<'a>(e: &'a DirEntry) -> &'a str {
  e.path().to_str().unwrap()
}

lazy_static::lazy_static! {
  pub static ref TEMPLATES: tera::Tera = {
    let mut tera = tera::Tera::default();

    let mut entries: Vec<DirEntry> = TEMPLATES_DIR
      .find("*").unwrap()
      .filter(|e| !file_name(e).starts_with('.') )
      .cloned().collect();
    entries.sort_by(|a,b|{
      let a_depth = path_name(a).matches("/").count();
      let b_depth = path_name(b).matches("/").count();
      let a_priority = file_name(a).starts_with("_");
      let b_priority = file_name(b).starts_with("_");

      if a_priority == b_priority {
        a_depth.cmp(&b_depth)
      } else {
        b_priority.cmp(&a_priority)
      }
    });

    for entry in entries {
      if let Some(file) = entry.as_file() {
        let pathname = path_name(&entry);

        if pathname.ends_with(".scss") {
          let style = grass::from_string(
            file.contents_utf8().expect(&format!("File is not utf-8: {}", pathname)).to_string(),
            &grass::Options::default().style(grass::OutputStyle::Compressed)
          ).expect(&format!("Failed to compile SCSS: {}", pathname));
          tera.add_raw_template(pathname, &style).expect("could not add template");
        } else {
          tera.add_raw_template(
            pathname,
            file.contents_utf8().expect(&format!("File is not utf-8: {}", pathname))
          ).expect("Could not add template");
        }
      }
    }

    tera
  };

  pub static ref TEMPLATE_NAMES: Vec<&'static str> = TEMPLATES.get_template_names().collect();
}

pub fn render(lang: Lang, template_name: &str, ctx: &Context) -> TeraResult<String> {
  let local_template = format!("{}.{}", template_name, lang.code());
  let template = if TEMPLATE_NAMES.contains(&local_template.as_str()) { &local_template } else { template_name };
  TEMPLATES.render(template, &ctx)
}

pub fn render_from_serialize<S: serde::Serialize>(lang: Lang, template_name: &str, o: &S) -> TeraResult<String> {
  render(lang, template_name, &Context::from_serialize(o)?)
}

pub struct HtmlWithLocale {
  pub content: String,
  pub lang: Lang,
}

impl HtmlWithLocale {
  pub fn from_context(renderer: &Renderer, lang: Lang, template_name: &str, ctx: &Context) -> TeraResult<Self> {
    Ok(Self{ lang, content: renderer.from_context(lang, template_name, ctx)? })
  }

  pub fn from_serialize<S: serde::Serialize>(renderer: &Renderer, lang: Lang, template_name: &str, s: &S) -> TeraResult<Self> {
    Ok(Self{ lang, content: renderer.from_serialize(lang, template_name, s)? })
  }

  pub fn no_context(renderer: &Renderer, lang: Lang, template_name: &str) -> TeraResult<Self> {
    Ok(Self{ lang, content: renderer.no_context(lang, template_name)? })
  }
}

impl<'r> response::Responder<'r, 'static> for HtmlWithLocale {
  fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
    response::Response::build()
      .sized_body(self.content.len(), std::io::Cursor::new(self.content))
      .raw_header("Content-Language", self.lang.code())
      .header(ContentType::new("text", "html"))
      .ok()
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Lang {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    Outcome::Success(Lang::find_in_request_headers(&req).unwrap_or(Lang::En) )
  }
}

pub struct MaybeLang {
  pub value: Option<Lang> 
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MaybeLang {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    Outcome::Success(MaybeLang{ value: Lang::find_in_request_headers(&req) })
  }
}

