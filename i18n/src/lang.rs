use super::*;

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

  pub fn detect_from_text(text: &str) -> Self {
    use whatlang::{Detector, Lang as L};
    if let Some(L::Spa) = Detector::with_allowlist(vec![L::Eng, L::Spa]).detect_lang(text) {
      Self::Es
    } else {
      Self::En
    }
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

