use crate::{
  models::{Document, Proof},
  Base64Standard, Error, Result,
};
use serde::Serialize;
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PreviewPart {
  size_in_bytes: usize,
  content_type: String,
  friendly_name: String,
  is_base: bool,
  #[serde(with = "Base64Standard")]
  payload: Vec<u8>,
}

impl PreviewPart {
  fn new(payload: &[u8], media_type: &str, filename: &str, is_base: bool) -> Self {
    PreviewPart{
      size_in_bytes: payload.len() ,
      content_type: media_type.to_string(),
      friendly_name: filename.to_string(),
      is_base,
      payload: payload.to_vec(),
    }
  }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Previewer {
  parts: Vec<PreviewPart>,
  has_kyc_endorsement: bool,
}

impl Previewer {
  pub fn create(payload: &[u8], has_kyc_endorsement: bool) -> Result<Self> {
    let mime_and_ext = Document::mime_and_ext(payload, None);
    let mut parts = match (mime_and_ext.0.as_str(), mime_and_ext.1.as_str()) {
      ("application/zip", _) => Self::index_as_zip(payload)?,
      (media_type, ext) => vec![PreviewPart::new(payload, media_type, &format!("document.{ext}"), true)],
    };
    
    use std::cmp::Ordering;
    let numbered = regex::Regex::new(r"^\d{1,3}[-_].*").unwrap();

    parts.sort_by(|a,b|{
      if a.is_base { return Ordering::Less }
      if b.is_base { return Ordering::Greater }

      match (numbered.is_match(&a.friendly_name), numbered.is_match(&b.friendly_name)) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (true, true) => a.friendly_name.cmp(&b.friendly_name),
        (false, false) => Proof::type_priority(&a.content_type).cmp(&Proof::type_priority(&b.content_type)),
      }
    });

    Ok(Self{ parts, has_kyc_endorsement })
  }

  fn index_as_zip(payload: &[u8]) -> Result<Vec<PreviewPart>> {
    use std::io::Read;
    let mut parts = vec![PreviewPart::new(payload, "application/zip", "full_zip_file", true)];

    let cursor = std::io::Cursor::new(payload);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
      let (friendly_name, bytes) = {
        let mut file = archive.by_index(i)?;
        if !file.is_file() {
          continue;
        }

        let mut buffer: Vec<u8> = vec![];
        if file.read_to_end(&mut buffer).is_err() {
          continue;
        }

        match file.enclosed_name() {
          Some(name) => (name.to_string_lossy().to_string(), buffer),
          None => {
            return Err(Error::validation( "payload", &format!("file {} was malformed", i),))
          }
        }
      };

      let (mime, _) = Document::mime_and_ext(&bytes, Some(&friendly_name));

      parts.push(PreviewPart::new(&bytes, &mime, &friendly_name, false));
    }

    Ok(parts)
  }

  pub fn render_html(&self, lang: i18n::Lang) -> Result<String> {
    Ok(crate::RENDERER.render_localized_and_serialized("previewer", &std::path::PathBuf::from("preview.html"), lang, i18n::Lang::En, &self)?.inner_to_utf8()?)
  }
}
