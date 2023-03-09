use crate::{
  models::{
    storable::*,
    Person,
    Request,
    certos::*,
  },
  Result as CrateResult,
  Error
};
use csv;
use std::collections::HashMap;

pub enum ImageOrText {
  Image(Vec<u8>),
  Text(String),
}

pub enum WizardTemplate {
  Existing{ template_id: i32 },
  New {
    kind: TemplateKind,
    logo: ImageOrText,
    name: String,
  }
}

impl WizardTemplate {
  pub async fn get_template_id(self, person: &Person) -> CrateResult<i32> {
    let lang = person.attrs.lang;
    let org = person.org().await?;
    let app = org.get_or_create_certos_app().await?;

    let id = match self {
      WizardTemplate::Existing{ template_id } => {
        org.template_scope().id_eq(template_id).one().await?.attrs.id
      },
      WizardTemplate::New { logo, kind, name } => {
        let (custom_message, payload) = WizardTemplate::make_template_zip(lang, logo, kind).await?;

        person.state.template()
          .insert(InsertTemplate{
            app_id: app.attrs.id,
            person_id: person.attrs.id,
            org_id: org.attrs.id,
            name: name,
            kind: kind,
            schema: serde_json::to_string(&kind.default_schema())?,
            og_title_override: None,
            custom_message: Some(custom_message),
            size_in_bytes: payload.len() as i32,
          }).validate_and_save(&payload).await?.attrs.id
      }
    };

    Ok(id)
  }

  pub async fn make_template_zip(lang: i18n::Lang, logo: ImageOrText, kind: TemplateKind) -> CrateResult<(String, Vec<u8>)> {
    use std::io::Write;
    use zip::write::FileOptions;

    let (filename, custom_message) = match kind {
      TemplateKind::Diploma => ("diploma", i18n::t!(lang, template_message_for_diploma)),
      TemplateKind::Attendance => ("attendance", i18n::t!(lang, template_message_for_attendance)),
      TemplateKind::Invitation => ("invitation", i18n::t!(lang, template_message_for_invitation)),
    };

    let mut context = i18n::Context::new();
    match logo {
      ImageOrText::Image(i) => {
        let mime = tree_magic_mini::from_u8(&i);
        if !mime.starts_with("image/") {
          return Err(Error::validation("logo_image", "not_a_valid_image_file"));
        }
        context.insert("image", &format!("data:{};base64,{}",mime,base64::encode(&i)));
      },
      ImageOrText::Text(n) => context.insert("issuer", &n),
    };
    let html = i18n::render(lang, &format!("template_builder/{filename}.html.tera"), &context)?;

    let mut file = vec![];
    {
      let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut file));
      zip.start_file(format!("{filename}.html.tera"), FileOptions::default())?;
      zip.write_all(html.as_bytes())?;
      zip.flush()?;
      zip.finish()?;
    }
    Ok((custom_message, file))
  }
}

pub struct Wizard {
  pub person: Person,
  pub template: WizardTemplate,
  pub name: String,
  pub csv: Vec<u8>,
}

impl Wizard {
  pub async fn process(self) -> CrateResult<Request> {
    let org = self.person.org().await?;
    let app = org.get_or_create_certos_app().await?;

    let template_id = self.template.get_template_id(&self.person).await?;

    let sanitized = if let Err(std::str::Utf8Error{..}) = std::str::from_utf8(&self.csv) {
      Some(self.csv.iter().map(|b| *b as char).collect::<String>().into_bytes())
    } else {
      None
    };

    let reader_buffer: &[u8] = &sanitized.as_deref().unwrap_or(&self.csv);

    let mut rows = Wizard::read_csv_from_payload(reader_buffer).await;

    for header in rows.headers()? {
      if !header.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(Error::validation("payload", "non_ascii_character"));
      }
    }
    
    let mut params: Vec<HashMap<String,String>> = vec![];

    for result in rows.deserialize() {
      match result {
        Err(error) => {
          let err = match error.kind() {
            csv::ErrorKind::UnequalLengths{..} => "unequal_lengths",
            csv::ErrorKind::Utf8 {..} => "utf8",
            _ => "unexpected",
          };
          return Err(Error::validation("payload", err));
        },
        Ok(p) => params.push(p)
      }
    }

    let request = self.person.state.request()
      .insert(InsertRequest{
        app_id: app.attrs.id,
        person_id: self.person.attrs.id,
        org_id: org.attrs.id,
        template_id,
        state: "received".to_string(),
        name: self.name,
        size_in_bytes: self.csv.len() as i32,
      }).save().await?;

    request.storage_put(&reader_buffer).await?;

    let received = request.in_received()?;
    received.append_entries(&params).await?;

    Ok(request)
  }

  pub async fn read_csv_from_payload(reader_buffer: &[u8]) -> csv::Reader<&[u8]> {
    let separator = if String::from_utf8_lossy(reader_buffer).contains(",") {
      b','
    } else {
      b';'
    };
    csv::ReaderBuilder::new().delimiter(separator).from_reader(reader_buffer)
  }
}

pub struct JsonIssuanceBuilder {
  pub person: Person,
  pub template: WizardTemplate,
  pub name: String,
  pub entries: Vec<HashMap<String,String>>,
}

impl JsonIssuanceBuilder {
  pub async fn process(self) -> CrateResult<request::Received> {
    let org = self.person.org().await?;
    let app = org.get_or_create_certos_app().await?;

    let template_id = self.template.get_template_id(&self.person).await?;

    for e in &self.entries {
      for k in e.keys() {
        if !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
          return Err(Error::validation("payload", "non_ascii_character"));
        }
      }
    }
    
    let received = self.person.state.request()
      .insert(InsertRequest{
        app_id: app.attrs.id,
        person_id: self.person.attrs.id,
        org_id: org.attrs.id,
        template_id,
        state: "received".to_string(),
        name: self.name,
        size_in_bytes: 0,
      }).save().await?.in_received()?;

    received.append_entries(&self.entries).await?;

    Ok(received)
  }
}
