pub mod error;
pub mod renderer;
pub mod translations;

pub use tera::{self, Context, Tera, Result as TeraResult};
pub use grass;
pub use include_dir::{self, Dir, DirEntry};
pub use formatx;
pub use renderer::Renderer;
use std::borrow::Cow;
pub use lazy_static;

use rocket::{
  self,
  response,
  http::{ContentType, hyper::header},
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
}

static TEMPLATES_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/templates");
lazy_static::lazy_static! {
  static ref RENDERER: Renderer<&'static Dir<'static>> = Renderer::new(&TEMPLATES_DIR).unwrap();
}


pub fn render(lang: Lang, template_name: &str, ctx: &Context) -> TeraResult<String> {
  let local_template = format!("{}.{}", template_name, lang.code());
  let template_names: Vec<&str> = RENDERER.htmls.get_template_names().collect();
  let template = if template_names.contains(&local_template.as_str()) { &local_template } else { template_name };
  RENDERER.htmls.render(template, &ctx)
}

pub fn render_from_serialize<S: serde::Serialize>(lang: Lang, template_name: &str, o: &S) -> TeraResult<String> {
  render(lang, template_name, &Context::from_serialize(o)?)
}

pub struct HtmlWithLocale {
  pub content: String,
  pub lang: Lang,
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

pub struct LocalizedResponse<'a> {
  pub inner: Cow<'a, [u8]>,
  pub content_type: ContentType,
  pub content_language: Lang,
}

impl<'a> LocalizedResponse<'a> {
  pub fn new(inner: Cow<'a, [u8]>, content_type: ContentType, content_language: Lang) -> LocalizedResponse {
    Self { inner, content_type, content_language }
  }

  pub fn inner_to_utf8(self) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(self.inner.into_owned())
  }

  pub fn into_owned<'b>(&'a self) -> LocalizedResponse<'b> {
    LocalizedResponse {
      inner: Cow::Owned(self.inner.clone().into_owned()),
      content_type: self.content_type.clone(),
      content_language: self.content_language
    }
  }
}

impl<'r> response::Responder<'r, 'r> for LocalizedResponse<'r> {
  fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
    response::Response::build()
      .sized_body(self.inner.len(), std::io::Cursor::new(self.inner))
      .raw_header(header::CONTENT_LANGUAGE.as_str(), self.content_language.code())
      .header(self.content_type)
      .ok()
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Lang {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    Outcome::Success(MaybeLang::from_request_base(&req).value.unwrap_or(Lang::En) )
  }
}

pub struct MaybeLang {
  pub value: Option<Lang> 
}

impl MaybeLang {
  pub fn from_request_base(r: &Request) -> Self {
    let value = r.headers()
      .get_one("Accept-Language")
      .unwrap_or("en")
      .split(",")
      // Get the locale, not the country code
      .filter_map(|l| l.split(|c| c == '-' || c == ';').nth(0))
      // Get the first requested locale we support
      .find(|l| *l == "en" || *l == "es" )
      .map(|l| if l == "es" { Lang::Es } else { Lang::En });

    Self { value }
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MaybeLang {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    Outcome::Success(Self::from_request_base(&req))
  }
}

