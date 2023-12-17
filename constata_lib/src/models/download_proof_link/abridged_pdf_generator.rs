use super::{
  *,
  document::Document,
  template_kind::TemplateKind,
};
use i18n::{Lang, renderer::RendererFs};
use std::io::BufWriter;
use std::path::Path;
use qrcode_generator::QrCodeEcc;
use printpdf::{
  *,
  lopdf::{
    StringFormat::Literal,
    Dictionary,
    Object
  }
};

pub struct AbridgedPdfGenerator {
}

impl AbridgedPdfGenerator {
  pub async fn generate(link: &DownloadProofLink, l: Lang) -> ConstataResult<Vec<u8>> {
    let document = link.document().await?;
    let bulletin = document.in_accepted()?.bulletin().await?.in_published()?;
    let url = link.public_certificate_url();

    let (title, verification_call_to_action, fields) = Self::values_based_on_template_kind_and_schema(&document, l).await?;
    let signers = Self::signers(&document, l).await?;

    let height = Mm(f64::max(
      300.0,
      (65 + (25 * signers.len()) + (18 * fields.len()) + 30) as f64,
    ));

    let (doc, page_ref, layer_ref) = PdfDocument::new(&title, Mm(215.0), height, "Main");
    let page = doc.get_page(page_ref);
    let current_layer = page.get_layer(layer_ref);
    let inter = doc.add_external_font(&*crate::RENDERER.fs.read(Path::new("fonts/InterTight-Light.ttf"))?)?;
    let manrope = doc.add_external_font(&*crate::RENDERER.fs.read(Path::new("fonts/Manrope-ExtraBold.ttf"))?)?;

    Self::add_link_annotation(page, height, &url);

    let b = LayerBuilder::new(current_layer, height, manrope, inter);

    b.write_title(&title, 27.0, -2.0);
    b.br();
    b.br();

    b.write_text(&i18n::t!(l, abridged_lead_text), 13.0, 20.0, 200);
    b.write_text(&verification_call_to_action, 13.0, 20.0, 200);
    b.write_link(&url);
    b.br();
    b.br();

    for signer in signers {
      b.write_subtitle(&i18n::t!(l, abridged_signed_by));
      b.write_text(&signer, 11.0, 15.0, 95);
      b.br();
    }

    b.write_subtitle(&i18n::t!(l, abridged_stamped_on));
    b.write_text(&bulletin.block_time().to_rfc2822(), 11.0, 15.0, 95);
    b.br();

    for (label, value) in fields {
      b.write_subtitle(&label);
      b.write_text(&value, 20.0, 22.0, 55);
      b.br();
    }
    b.done();
    b.add_qr_code(&url)?;

    let mut writer = BufWriter::new(vec![]);
    doc.save(&mut writer)?;
    Ok(writer.into_inner()?)
  }

  async fn signers(document: &Document, l: Lang) -> ConstataResult<Vec<String>> {
    let mut signers: Vec<String> = vec![];
    for part in document.document_part_vec().await?.into_iter() {
      for sig in &part.document_part_signature_vec().await? {
        if let Some(endorsement) = sig.pubkey().await?.person().await?.endorsement_string(l, false).await? {
          signers.push(endorsement);
        }
      }
    }
    Ok(signers)
  }

  async fn values_based_on_template_kind_and_schema(document: &Document, l: Lang) -> ConstataResult<(String, String, Vec<(String, String)>)> {
    let tuple = match document.entry_optional().await? {
      Some(entry) => {
        let (title, verification_call_to_action) = match entry.template_kind().await? {
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

        let schema = entry.issuance().await?.template().await?.parsed_schema()?;
        let params = entry.parsed_params()?;

        let mut fields = vec![];

        for field in &schema {
          if field.name == "email" { continue; }
          let Some(value) = params.get(&field.name) else { continue };
          if value.is_empty() { continue; }
          fields.push(( field.i18n_label(l).unwrap_or(&field.name).to_string(), value.to_string() ));
        }

        (title, verification_call_to_action, fields)
      },
      None => (
        i18n::t!(l, abridged_title_default),
        i18n::t!(l, abridged_verify_default),
        vec![],
      )
    };

    Ok(tuple)
  }

  fn add_link_annotation(page: PdfPageReference, height: Mm, url: &str) {
    let action = Dictionary::from_iter(vec![
      ("Type", "Action".into()),
      ("S", Object::Name(b"URI".to_vec())),
      ("URI", Object::String(url.to_string().into_bytes(), Literal)),
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
        ("Contents", Object::String(url.to_string().into_bytes(), Literal)),
        ("A", action.into()),
    ]);

    let annotations = Dictionary::from_iter(vec![
        ("Annots", Object::Array(vec![annotation.into()]))
    ]);

    page.extend_with(annotations);
  }
}

struct LayerBuilder {
  layer: PdfLayerReference,
  height: Mm,
  manrope: IndirectFontRef,
  inter: IndirectFontRef,
}

impl LayerBuilder {
  fn new(layer: PdfLayerReference, height: Mm, manrope: IndirectFontRef, inter: IndirectFontRef) -> Self {
    let builder = Self { layer, height, manrope, inter };
    builder.layer.begin_text_section();
    builder.layer.set_text_cursor(Mm(10.0), height - Mm(8.0));
    builder
  }

  fn write_title(&self, text: &str, size: f64, spacing: f64) {
    self.layer.set_fill_color(Color::Rgb(Rgb::new(0.0,0.0,0.0, None)));
    self.layer.set_font(&self.manrope, size);
    self.layer.set_line_height(size);
    self.layer.set_character_spacing(spacing);
    self.layer.add_line_break();
    self.layer.write_text(text, &self.manrope);
  }

  fn write_subtitle(&self, text: &str) {
    self.write_title(&format!("{}:", text), 11.0, -0.5);
  }
  
  fn write_text(&self, text: &str, size: f64, line_height: f64, wrap_size: usize) {
    self.layer.set_fill_color(Color::Rgb(Rgb::new(0.0,0.0,0.0, None)));
    self.layer.set_character_spacing(0.3);
    self.layer.set_line_height(line_height);
    self.layer.set_font(&self.inter, size);
    for line in Self::wrap(&text, wrap_size) {
      self.layer.add_line_break();
      self.layer.write_text(line, &self.inter);
    }
  }

  fn write_link(&self, url: &str) {
    self.layer.set_fill_color(Color::Rgb(Rgb::new(0.0,0.0,255.0, None)));
    self.layer.set_font(&self.inter, 11.0);
    self.layer.add_line_break();
    self.layer.write_text(url, &self.inter);
  }

  fn br(&self) {
    self.layer.set_line_height(15.0);
    self.layer.add_line_break();
  }

  fn done(&self) {
    self.layer.end_text_section();
  }

  fn wrap(s: &str, size: usize) -> Vec<String> {
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
  }

  fn add_qr_code(&self, url: &str) -> ConstataResult<()> {
    let qr: String = qrcode_generator::to_svg_to_string(&url, QrCodeEcc::Low, 300, None::<&str>)?;
    let svg = Svg::parse(&qr)?;
    let transform = SvgTransform {
      translate_x: Some(Mm(182.5).into()),
      translate_y: Some(Pt(self.height.into_pt().0 - 148.0)),
      ..Default::default()
    };
    svg.into_xobject(&self.layer).add_to_layer(&self.layer, transform);
    Ok(())
  }
}


