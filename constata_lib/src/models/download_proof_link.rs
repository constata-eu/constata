/* ToDo:
 *  Refactor template kind macro usage.
 *  Refactor PDF generation.
*/

use super::{*, blockchain::PrivateKey};
use crate::{
  models::{
    template_kind::TemplateKind,
  },
  Site,
  Result,
};
use chrono::Utc;


model!{
  state: Site,
  table: download_proof_links,
  struct DownloadProofLink {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    access_token_id: i32,
    #[sqlx_model_hints(varchar)]
    document_id: String,
    #[sqlx_model_hints(varchar)]
    public_token: String,
    #[sqlx_model_hints(timestamptz, default)]
    published_at: Option<UtcDateTime>,
    #[sqlx_model_hints(boolean, default)]
    admin_visited: bool,
    #[sqlx_model_hints(int4, default)]
    public_visit_count: i32,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  queries {
    active(
      "deletion_id IS NULL AND access_token_id = 
      (SELECT id FROM access_tokens WHERE token = $1 AND NOT expired AND kind='download_proof_link')
      ", token: String
    ),
    active_by_document_id(
      "deletion_id IS NULL AND document_id = $1 AND
        EXISTS (SELECT id FROM access_tokens WHERE id = access_token_id AND NOT expired)
      ", document_id: String
    ),
    public_certificate_active(
      "deletion_id IS NULL AND public_token = $1 AND published_at IS NOT NULL", token: String
    ),
  },
  belongs_to {
    Document(document_id),
    AccessToken(access_token_id),
    OrgDeletion(deletion_id),
  }
}

impl DownloadProofLink {
  pub async fn token(&self) -> Result<String> {
    Ok(self.access_token().await?.attrs.token)
  }                              

  pub async fn valid_until(&self) -> Result<Option<UtcDateTime>> {
    Ok(self.access_token().await?.attrs.auto_expires_on)
  }

  pub async fn org(&self) -> sqlx::Result<Org> {
    self.document().await?.org().await
  }

  pub async fn entry_optional(&self) -> sqlx::Result<Option<Entry>> {
    self.document().await?.entry_optional().await
  }    

  pub async fn html_proof(&self, key: &PrivateKey, lang: i18n::Lang) -> Result<String> {
    self.document().await?
      .story().await?
      .proof(self.state.settings.network, &key).await?
      .render_html(lang)
  }

  pub async fn full_url(&self) -> Result<String> {
    Ok(format!("{}/download-proof/{}", &self.state.settings.url, self.token().await?))
  }

  pub async fn safe_env_url(&self) -> Result<String> {
    Ok(format!("{}/safe/{}", &self.state.settings.url, self.token().await?))
  }

  pub fn public_certificate_url(&self) -> String {
    format!("{}/certificate/{}", &self.state.settings.url, self.public_token())
  }

  pub async fn make_access_token_eternal(&self) -> sqlx::Result<AccessToken> {
    self.access_token().await?.update().auto_expires_on(None).save().await
  }

  pub async fn publish(&self) -> sqlx::Result<DownloadProofLink> {
    self.make_access_token_eternal().await?;
    self.clone().update().published_at(Some(Utc::now())).save().await
  }

  pub async fn unpublish(&self) -> sqlx::Result<DownloadProofLink> {
    self.clone().update().published_at(None).save().await
  }

  pub async fn set_visited(&self) -> sqlx::Result<DownloadProofLink> {
    self.clone().update().admin_visited(true).save().await
  }

  pub async fn update_public_visit_count(&self) -> sqlx::Result<DownloadProofLink> {
    self.clone().update().public_visit_count(*self.public_visit_count() + 1).save().await
  }

  pub async fn image_url(&self) -> Result<String> {
    let image = match self.org().await?.attrs.logo_url {
      Some(url) => url,
      None => self.state.settings.default_logo_url().to_string(),
    };

    Ok(image)
  }

  pub async fn title(&self) -> Result<Option<String>> {
    if let Some(entry) = self.entry_optional().await? {
     entry.title().await
    } else {
      Ok(None)
    }
  }
  pub async fn template_kind(&self) -> Result<Option<TemplateKind>> {
    Ok(match self.entry_optional().await? {
      Some(e) => Some(e.template_kind().await?),
      None => None,
    })
  }

  pub async fn share_on_social_networks_call_to_action(&self, l: &i18n::Lang) -> Result<String> {
    let share_on_social_networks_call_to_action = match self.document().await?.entry_optional().await? {
      Some(entry) => {
        match entry.template_kind().await? {
          TemplateKind::Diploma => i18n::t!(l, public_certificate_share_text_diploma),
          TemplateKind::Attendance => i18n::t!(l, public_certificate_share_text_attendance),
          TemplateKind::Badge => i18n::t!(l, public_certificate_share_text_badge),
        }
      },
      None => i18n::t!(l, public_certificate_share_text_default),
    };

    Ok(share_on_social_networks_call_to_action)
  }

  pub async fn abridged_pdfs_zip(&self, l: i18n::Lang) -> Result<(String, Vec<u8>)> {
    use std::io::Write;
    use zip::write::FileOptions;

    let mut destination_buffer = vec![];

    {
      let mut destination = zip::ZipWriter::new(std::io::Cursor::new(&mut destination_buffer));
      destination.start_file("español.pdf", FileOptions::default())?;
      destination.write_all(&self.abridged_pdf(i18n::Lang::Es).await?)?;
      destination.flush()?;
      destination.start_file("english.pdf", FileOptions::default())?;
      destination.write_all(&self.abridged_pdf(i18n::Lang::En).await?)?;
      destination.flush()?;
      destination.finish()?;
    }

    let filename = match self.document().await?.entry_optional().await? {
      Some(entry) => {
        match entry.template_kind().await? {
          TemplateKind::Diploma => i18n::t!(l, abridged_diploma_zip_name),
          TemplateKind::Attendance => i18n::t!(l, abridged_attendance_zip_name),
          TemplateKind::Badge => i18n::t!(l, abridged_badge_zip_name),
        }
      },
      None => i18n::t!(l, abridged_document_zip_name),
    };


    Ok((filename, destination_buffer))
  }

  pub async fn abridged_pdf(&self, l: i18n::Lang) -> Result<Vec<u8>> {
    use i18n::renderer::RendererFs;
    use std::io::BufWriter;

    use printpdf::{
      *,
      lopdf::{
        StringFormat::Literal,
        Dictionary,
        Object
      }
    };
    use std::path::Path;
    use qrcode_generator::QrCodeEcc;

    let document = self.document().await?;

    let bulletin = document.in_accepted()?.bulletin().await?.in_published()?;

    let url = self.public_certificate_url();

    /* Agregar fecha de sellado de tiempo */

    let (title, verify_text, fields) = match document.entry_optional().await? {
      Some(entry) => {
        let (t, v) = match entry.template_kind().await? {
          TemplateKind::Diploma => (
            i18n::t!(l, abridged_title_diploma),
            i18n::t!(l, abridged_verify_diploma),
          ),
          TemplateKind::Attendance => (
            i18n::t!(l, abridged_title_attendance),
            i18n::t!(l, abridged_verify_attendance),
          ),
          TemplateKind::Badge => (
            i18n::t!(l, abridged_title_badge),
            i18n::t!(l, abridged_verify_badge),
          ),
        };

        let schema = entry.request().await?.template().await?.parsed_schema()?;
        let params = entry.parsed_params()?;

        let mut f = vec![];

        for field in &schema {
          if field.name == "email" { continue; }
          let Some(value) = params.get(&field.name) else { continue };
          if value.is_empty() { continue; }
          f.push(( field.i18n_label(l).unwrap_or(&field.name).to_string(), value.to_string() ));
        }

        (t, v, f)
      },
      None => (
        i18n::t!(l, abridged_title_default),
        i18n::t!(l, abridged_verify_default),
        vec![],
      )
    };

    let mut signers: Vec<String> = vec![];
    for part in document.document_part_vec().await?.into_iter() {
      for sig in &part.document_part_signature_vec().await? {
        if let Some(endorsement) = sig.pubkey().await?.person().await?.endorsement_string(l, false).await? {
          signers.push(endorsement);
        }
      }
    }

    let calc_height = (65 + (25 * signers.len()) + (21 * fields.len()) + 20) as f64;

    let height = Mm(f64::max(calc_height, 300.0));

    let (doc, page1, layer1) = PdfDocument::new(&title, Mm(215.0), height, "Main");

    let page = doc.get_page(page1);
    let current_layer = page.get_layer(layer1);
    let inter = doc.add_external_font(&*crate::RENDERER.fs.read(Path::new("fonts/InterTight-Light.ttf"))?)?;
    let manrope = doc.add_external_font(&*crate::RENDERER.fs.read(Path::new("fonts/Manrope-ExtraBold.ttf"))?)?;

    let action = Dictionary::from_iter(vec![
      ("Type", "Action".into()),
      ("S", Object::Name(b"URI".to_vec())),
      ("URI", Object::String(url.clone().into_bytes(), Literal)),
    ]);

    let annotation = Dictionary::from_iter(vec![
        ("Type", "Annot".into()),
        ("Subtype", Object::Name(b"Link".to_vec())),
        ("Rect", vec![
          20.into(), // Left
          (height.into_pt().0 - 123.0).into(), // Top
          500.into(), // Right
          (height.into_pt().0 - 145.0).into() // Bottom
        ].into()),
        ("C", vec![].into()),
        ("Contents", Object::String(url.clone().into_bytes(), Literal)),
        ("A", action.into()),
    ]);

    let annotations = Dictionary::from_iter(vec![
        ("Annots", Object::Array(vec![annotation.into()]))
    ]);

    page.extend_with(annotations);

    let qrcode_string: String = qrcode_generator::to_svg_to_string(&url, QrCodeEcc::Low, 300, None::<&str>).unwrap();
    let qr_svg = Svg::parse(&qrcode_string)?;
    let qr_transform = SvgTransform {
      translate_x: Some(Mm(182.5).into()),
      translate_y: Some(Pt(height.into_pt().0 - 148.0)),
      ..Default::default()
    };
    let reference_two = qr_svg.into_xobject(&current_layer);
    reference_two.add_to_layer(&current_layer, qr_transform);

    current_layer.begin_text_section();
    /* Title */
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0,0.0,0.0, None)));
    current_layer.set_font(&manrope, 27.0);
    current_layer.set_text_cursor(Mm(10.0), height - Mm(20.0));
    current_layer.set_character_spacing(-2.0);
    current_layer.write_text(&title, &manrope);

    current_layer.set_line_height(40.0);
    current_layer.add_line_break();

    /* Presentation label */
    current_layer.set_character_spacing(0.3);
    current_layer.set_line_height(20.0);
    current_layer.set_font(&inter, 13.0);
    current_layer.write_text(i18n::t!(l, abridged_lead_text), &inter);
    current_layer.add_line_break();
    current_layer.write_text(verify_text, &inter);
    current_layer.add_line_break();
    current_layer.set_font(&inter, 11.0);
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0,0.0,255.0, None)));
    current_layer.write_text(&url, &inter);
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0,0.0,0.0, None)));

    let wrap = |s: &str, size: usize| {
      let nospace = s.trim()
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
      let no_newline = nospace.trim()
        .split('\n')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
      let mut options = textwrap::Options::new(size);
      options.word_splitter = textwrap::WordSplitter::NoHyphenation;
      textwrap::wrap(&no_newline, options).iter().map(|i| i.to_string() ).collect::<Vec<String>>()
    };

    /* Signers */
    for signer in signers {
      current_layer.set_line_height(40.0);
      current_layer.add_line_break();

      /* Field label */
      current_layer.set_character_spacing(-0.5);
      current_layer.set_line_height(11.0);
      current_layer.set_font(&manrope, 11.0);
      current_layer.write_text(format!("{}:", i18n::t!(l, abridged_signed_by)), &manrope);

      /* Field value */
      current_layer.set_character_spacing(0.3);
      current_layer.set_font(&inter, 11.0);
      current_layer.set_line_height(15.0);
      for line in wrap(&signer, 95) {
        current_layer.add_line_break();
        current_layer.write_text(line, &inter);
      }
    }

    current_layer.set_line_height(30.0);
    current_layer.add_line_break();

    /* Published */
    /* Field label */
    current_layer.set_character_spacing(-0.5);
    current_layer.set_line_height(11.0);
    current_layer.set_font(&manrope, 11.0);
    current_layer.write_text(format!("{}:", i18n::t!(l, abridged_stamped_on)), &manrope);

    /* Field value */
    current_layer.set_character_spacing(0.3);
    current_layer.set_font(&inter, 11.0);
    current_layer.set_line_height(15.0);
    current_layer.add_line_break();
    current_layer.write_text(bulletin.block_time().to_rfc2822(), &inter);

    for (label, value) in fields {
      current_layer.set_line_height(30.0);
      current_layer.add_line_break();

      /* Field label */
      current_layer.set_character_spacing(-0.5);
      current_layer.set_line_height(11.0);
      current_layer.set_font(&manrope, 11.0);
      current_layer.write_text(format!("{}:", label), &manrope);

      /* Field value */
      current_layer.set_character_spacing(0.0);
      current_layer.set_font(&inter, 20.0);
      current_layer.set_line_height(22.0);
      for line in wrap(&value, 55) {
        current_layer.add_line_break();
        current_layer.write_text(line, &inter);
      }
    }

    current_layer.end_text_section();

    let mut writer = BufWriter::new(vec![]);
    doc.save(&mut writer)?;
    Ok(writer.into_inner()?)
  }
}

impl InsertDownloadProofLink {
  pub async fn new(document: &Document, duration_days: i64) -> Result<Self> {
    let org = document.org().await?;
    let person = org.admin().await?;
    let access_token = org.state.access_token().create(&person, AccessTokenKind::DownloadProofLink, duration_days).await?;

    Ok(Self{
      document_id: document.attrs.id.clone(),
      access_token_id: *access_token.id(),
      public_token: MagicLink::make_random_token(),
    })
  }
}


describe! {
  use std::collections::HashMap;
  regtest!{ create_public_certificate_and_switch_state (_db, c, mut chain)
    let alice = c.alice().await;
    let entry = alice.make_entry_and_sign_it().await;
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    let doc = entry.reloaded().await?.document().await?.expect("to get entry's document");
    let download_proof_link = alice.make_download_proof_link_from_doc(&doc, 30).await;
    let access_token = download_proof_link.access_token().await?;

    download_proof_link.publish().await?;
    assert_that!(download_proof_link.reloaded().await?.published_at().is_some());
    assert_that!(access_token.reloaded().await?.auto_expires_on().is_none());

    download_proof_link.unpublish().await?;
    assert_that!(download_proof_link.reloaded().await?.published_at().is_none());
    assert_that!(access_token.reloaded().await?.auto_expires_on().is_none());

    download_proof_link.publish().await?;
    assert_that!(download_proof_link.reloaded().await?.published_at().is_some());
    assert_that!(access_token.reloaded().await?.auto_expires_on().is_none());

    assert_eq!(
      download_proof_link.share_on_social_networks_call_to_action(&i18n::Lang::En).await?,
      "This diploma is certified by the Bitcoin blockchain!".to_string()
    );

    let template = entry.request().await?.template().await?;
    template.clone().update().kind(TemplateKind::Attendance).save().await?;
    assert_eq!(
      download_proof_link.share_on_social_networks_call_to_action(&i18n::Lang::En).await?,
      "This certificate of attendance is sealed by the Bitcoin blockchain!".to_string()
    );

    template.update().kind(TemplateKind::Badge).save().await?;
    assert_eq!(
      download_proof_link.share_on_social_networks_call_to_action(&i18n::Lang::En).await?,
      "This badge is certified by the Bitcoin blockchain!".to_string()
    );
  }

  regtest!{ gets_abridged_pdf_version (_db, c, mut chain)
    let alice = c.alice().await;
    alice.make_kyc_endorsement().await;
    let entry = alice.make_entry_and_sign_it().await;
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    let doc = entry.reloaded().await?.document().await?.expect("to get entry's document");
    let download_proof_link = alice.make_download_proof_link_from_doc(&doc, 30).await;
    download_proof_link.publish().await?;

    std::fs::write("../target/artifacts/test_es.pdf", &download_proof_link.abridged_pdf(i18n::Lang::Es).await?)?;
    std::fs::write("../target/artifacts/test_en.pdf", &download_proof_link.abridged_pdf(i18n::Lang::En).await?)?;
    let (filename, bytes) = &download_proof_link.abridged_pdfs_zip(i18n::Lang::Es).await?;
    std::fs::write(&format!("../target/artifacts/{}.zip", filename), bytes)?;
    assert_eq!(filename, "Diploma abreviado en inglés y español"); 
  }

  regtest!{ public_certificate_metadata (_db, c, mut chain)
    let alice = c.alice().await;
    let entry = alice.make_entry_and_sign_it().await;
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    let doc = entry.reloaded().await?.document().await?.expect("to get entry's document");
    let download_proof_link = alice.make_download_proof_link_from_doc(&doc, 30).await.publish().await?;

    assert_eq!(
      download_proof_link.title().await?,
      None
    );
    assert_eq!(
      download_proof_link.image_url().await?,
      "https://constata.eu/assets/images/logo.png".to_string()
    );

    let template = entry.request().await?.template().await?;
    let mut params: HashMap<String, String> = serde_json::from_str::<HashMap<String, String>>(entry.params())?;
    params.insert("motive".to_string(), "Curso de manejo".to_string());
    entry.clone().update().params(serde_json::to_string(&params).unwrap()).save().await?;
    assert_eq!(
      download_proof_link.title().await?.unwrap(),
      "Curso de manejo".to_string()
    );

    template.clone().update().og_title_override(Some("Curso de programación".to_string())).save().await?;
    alice.org().await.update().logo_url(Some("https://logodeprueba.com".to_string())).save().await?;
    assert_eq!(
      download_proof_link.title().await?.unwrap(),
      "Curso de programación".to_string()
    );
    assert_eq!(
      download_proof_link.reloaded().await?.image_url().await?,
      "https://logodeprueba.com".to_string()
    );
  }
}
