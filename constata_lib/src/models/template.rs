use super::*;
use std::io::Read;

model!{
  state: Site,
  table: templates,
  struct Template {
    #[sqlx_model_hints(int4, default)]
    id: i32,
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
    #[sqlx_model_hints(boolean, default)]
    archived: bool,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(template_kind)]
    kind: TemplateKind,
    schema: String,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  has_many {
    Issuance(template_id),
  },
  belongs_to {
    OrgDeletion(deletion_id),
  }
}

derive_storable!(Template, "wt");

impl Template {
  pub async fn payload(&self) -> ConstataResult<Vec<u8>> {
    self.storage_fetch().await
  }

  pub async fn read_name_and_bytes_from_payload(payload: &[u8]) -> ConstataResult<Vec<(String, Vec<u8>)>> {
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

  pub fn parsed_schema(&self) -> ConstataResult<TemplateSchema> {
    Ok(serde_json::from_str(self.schema())?)
  }
}

impl InsertTemplateHub {
  pub async fn validate_and_save(self, payload: &[u8]) -> ConstataResult<Template> {
    serde_json::from_str::<TemplateSchema>(self.schema()).map_err(|e| Error::validation("schema", &format!("{:?}", e)))?;

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
  dbtest!{ makes_template(_site, c)
    c.alice().await.make_template(read("template.zip")).await;
  }

  macro_rules! make_failing_template_test (
    ($test_name:ident, $file_name:expr, $schema:expr, $field:expr, $message:expr) => (
      dbtest! { $test_name (_site, c)
        let template_file = std::fs::read(&format!("src/test_support/resources/{}", $file_name))?;
        let result = c.alice().await.try_make_template(template_file, $schema).await;

        assert_that!(&result.unwrap_err(), structure![ Error::Validation {
          field: eq($field.to_string()),
          message: eq($message.to_string()),
        }])
      }
    )
  );

  make_failing_template_test![
    validates_tera_syntax,
    "invalid_tera_markup_template.zip",
    r#"[{"name":"alumno","optional":false,"common":false}]"#,
    "payload",
    "invalid_tera_syntax"
  ];

  make_failing_template_test![
    validates_tera_is_text,
    "invalid_tera_non_text.zip",
    r#"[{"name":"alumno","optional":false,"common":false}]"#,
    "payload",
    "invalid_tera_syntax"
  ];

  make_failing_template_test![
    validates_template_is_zip,
    "testvideo.webm",
    r#"[{"name":"alumno","optional":false,"common":false}]"#,
    "payload",
    "could_not_process_zip_file"
  ];

  make_failing_template_test![
    validates_template_schema,
    "template.zip",
    r#"[{"name":"alumno","optional":false,"common":false,"label":false}]"#,
    "schema",
    r#"Error("invalid type: boolean `false`, expected a string", line: 1, column: 63)"#
  ];
}
