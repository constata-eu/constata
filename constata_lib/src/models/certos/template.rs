use crate::{
  models::{
    model,
    Site,
    PersonId,
    UtcDateTime,
    OrgDeletion,
    TemplateKind,
    storable::*,
  },
  Error, Result,
};
use std::io::Read;

model!{
  state: Site,
  table: certos_templates,
  struct Template {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    app_id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(varchar)]
    name: String,
    #[sqlx_model_hints(varchar)]
    custom_message: Option<String>,
    #[sqlx_model_hints(varchar)]
    og_title_override: Option<String>,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    size_in_bytes: i32,
    #[sqlx_model_hints(template_kind)]
    kind: TemplateKind,
    schema: Option<String>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    OrgDeletion(deletion_id),
  }
}

derive_storable!(Template, "wt");

impl Template {
  pub async fn payload(&self) -> Result<Vec<u8>> {
    self.storage_fetch().await
  }

  pub async fn read_name_and_bytes_from_payload(payload: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let mut template_zip = zip::ZipArchive::new(std::io::Cursor::new(payload))?;
    let mut template_files = vec![];

    for i in 0..template_zip.len() {
      let mut file = template_zip.by_index(i)?;
      if !file.is_file() {
        continue;
      }

      let mut buffer = vec![];
      if file.read_to_end(&mut buffer).is_err() {
        continue;
      }

      let name = file.enclosed_name()
        .ok_or_else(|| Error::validation("file", "File name could not be read"))?
        .to_string_lossy()
        .to_string(); 

      template_files.push((name, buffer));
    }
    Ok(template_files)
  }

  pub fn is_tera(name: &str) -> Option<&str> {
    name.strip_suffix(".tera")
  }
}

impl InsertTemplateHub {
  pub async fn validate_and_save(self, payload: &[u8]) -> Result<Template> {
    let mut tera = i18n::tera::Tera::default();
    let names_and_bytes = Template::read_name_and_bytes_from_payload(&payload).await
      .map_err(|_| Error::validation("payload", "could_not_process_zip_file"))?;

    let invalid_tera = || Error::validation("payload", "invalid_tera_syntax");

    for (raw_name, bytes) in &names_and_bytes {
      if Template::is_tera(raw_name).is_some() {
        let utf8 = std::str::from_utf8(bytes).map_err(|_| invalid_tera())?;
        tera.add_raw_template(raw_name, utf8).map_err(|_| invalid_tera())?;
      }
    }

    let template = self.save().await?;
    template.storage_put(payload).await?;
    Ok(template)
  }
}

describe!{
  macro_rules! make_template_test (
    ($test_name:ident, $file_name:expr, $message:expr) => (
      dbtest! { $test_name (_site, c)
        let template_file = std::fs::read(&format!("src/test_support/resources/{}", $file_name))?;
        let result = c.alice().await.try_make_template(template_file).await;

        assert_that!(&result.unwrap_err(), structure![ Error::Validation {
          field: eq("payload".to_string()),
          message: eq($message.to_string()),
        }])
      }
    )
  );

  make_template_test![
    validates_tera_syntax,
    "invalid_tera_markup_template.zip",
    "invalid_tera_syntax"
  ];

  make_template_test![
    validates_tera_is_text,
    "invalid_tera_non_text.zip",
    "invalid_tera_syntax"
  ];

  make_template_test![
    validates_template_is_zip,
    "testvideo.webm",
    "could_not_process_zip_file"
  ];
}
