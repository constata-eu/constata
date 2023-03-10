use crate::{
  models::{
    Person,
    Request,
    certos::*,
  },
  Result as CrateResult,
  Error
};

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

pub struct Wizard {
  pub person: Person,
  pub template: WizardTemplate,
  pub name: String,
  pub csv: Vec<u8>,
}

impl Wizard {
  pub async fn process(self) -> CrateResult<Request> {
    let org = self.person.org().await?;
    let lang = self.person.attrs.lang;
    let app = org.get_or_create_certos_app().await?;

    let template_id = match self.template {
      WizardTemplate::Existing{ template_id } => {
        org.template_scope().id_eq(template_id).one().await?.attrs.id
      },
      WizardTemplate::New { logo, kind, name } => {
        let (custom_message, payload) = Wizard::make_template_zip(lang, logo, kind).await?;

        self.person.state.template()
          .insert(InsertTemplate{
            app_id: app.attrs.id,
            person_id: self.person.attrs.id,
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

    self.person.state.request()
      .insert(InsertRequest{
        app_id: org.get_or_create_certos_app().await?.attrs.id,
        person_id: self.person.attrs.id,
        org_id: org.attrs.id,
        template_id,
        state: "received".to_string(),
        name: self.name,
        size_in_bytes: self.csv.len() as i32,
      }).validate_and_save(&self.csv).await
  }

  pub async fn make_template_zip(lang: i18n::Lang, logo: ImageOrText, kind: TemplateKind) -> CrateResult<(String, Vec<u8>)> {
    use std::io::Write;
    use zip::write::FileOptions;

    let (filename, custom_message) = match kind {
      TemplateKind::Diploma => ("diploma", i18n::t!(lang, template_message_for_diploma)),
      TemplateKind::Attendance => ("attendance", i18n::t!(lang, template_message_for_attendance)),
      // To change when template is ready
      _ => ("invitation", i18n::t!(lang, template_message_for_badge)),
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


describe!{
  use std::io::Read;

  dbtest!{ submits_wizard_with_new_template(site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard {
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Image(read("wizard/logo.png")),
        kind: TemplateKind::Badge,
      },
      csv: read("wizard/default.csv"),
    };

    let request = w.process().await?;
    site.request().create_all_received().await?;

    assert_eq!(
      &request.entry_vec().await?[0].params_and_custom_message().await?.1.unwrap(),
      "Hola Stan Marsh, esta es una insignia por Arte con plastilina."
    );

    assert_eq!(request.entry_scope().count().await?, 2);
    std::fs::write("../target/artifacts/entry.zip", &request.entry_scope().one().await?.payload().await?)?;


    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(request.template().await?.payload().await?))?;
    let mut inner = zipfile.by_index(0)?;
    // To change
    assert_eq!(inner.name(), "invitation.html.tera");
    let mut contents = String::new();
    inner.read_to_string(&mut contents)?;
    assert_that!(&contents, rematch(r#"\{\{ name \}\}"#));
    assert_that!(&contents, rematch("data:image/png;base64,iVBORw0KG"));

    std::fs::write("../target/artifacts/template_from_submits_wizard_with_new_template.html", &contents)?;
  }

  test!{ creates_translated_templates
    assert_eq!(
      &Wizard::make_template_zip(i18n::Lang::Es, ImageOrText::Text("test".to_string()), TemplateKind::Diploma).await?.0,
      "Hola {{ name }}, este es tu diploma de {{ motive }}."
    );

    assert_eq!(
      &Wizard::make_template_zip(i18n::Lang::En, ImageOrText::Text("test".to_string()), TemplateKind::Badge).await?.0,
      "Hello {{ name }}, this is a badge for {{ motive }}."
    );
  }

  dbtest!{ submits_wizard_using_issuer_name(site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Text("City Wok".to_string()),
        kind: TemplateKind::Attendance,
      },
      csv: read("wizard/default.csv"),
    };

    let request = w.process().await?;
    site.request().create_all_received().await?;
    assert_eq!(request.entry_scope().count().await?, 2);
    std::fs::write("../target/artifacts/zip_from_submits_wizard_using_issuer_name_entry.zip", &request.entry_scope().one().await?.payload().await?)?;

    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(request.template().await?.payload().await?))?;
    let mut inner = zipfile.by_index(0)?;
    assert_eq!(inner.name(), "attendance.html.tera");
    let mut contents = String::new();
    inner.read_to_string(&mut contents)?;
    assert_that!(&contents, rematch(r#"\{\{ name \}\}"#));
    assert_that!(&contents, rematch("City Wok"));

    std::fs::write("../target/artifacts/template_from_submits_wizard_using_issuer_name.html", &contents)?;
  }

  dbtest!{ submits_wizard_using_unrecognized_image(_site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Image(read("wizard/default.csv")),
        kind: TemplateKind::Badge,
      },
      csv: read("wizard/default.csv"),
    };

    assert_that!(
      &w.process().await.unwrap_err(),
      structure![Error::Validation { field: rematch("logo_image"), message: rematch("not_a_valid_image_file") }]
    );
  }

  dbtest!{ submits_wizard_with_existing_template(site, c)
    let a = c.alice().await;
    let template = a.make_template(
      read("certos_template.zip"),
    ).await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::Existing {
        template_id: template.attrs.id,
      },
      csv: read("certos_request.csv"),
    };

    let request = w.process().await?;
    site.request().create_all_received().await?;
    assert_eq!(request.entry_scope().count().await?, 2);

    let bob = c.bob().await.person().await;
    let failing_wizard = Wizard{
      person: bob,
      name: "Bob's trying to use alice's template".to_string(),
      template: WizardTemplate::Existing {
        template_id: template.attrs.id,
      },
      csv: read("certos_request.csv"),
    };

    assert_that!(
      &failing_wizard.process().await.unwrap_err(),
      structure![Error::DatabaseError [is_variant!(sqlx::Error::RowNotFound)] ]
   );
  }
}
