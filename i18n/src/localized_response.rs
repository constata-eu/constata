use super::*;

pub struct LocalizedResponse<'a> {
  pub inner: Cow<'a, [u8]>,
  pub content_type: ContentType,
  pub content_language: Lang,
}

impl<'a> LocalizedResponse<'a> {
  pub fn new(inner: Cow<'a, [u8]>, content_type: ContentType, content_language: Lang) -> LocalizedResponse {
    Self { inner, content_type, content_language }
  }

  pub fn to_utf8(self) -> Result<String, std::string::FromUtf8Error> {
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

