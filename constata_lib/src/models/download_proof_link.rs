mod abridged_pdf_generator;
use abridged_pdf_generator::AbridgedPdfGenerator;
use super::{*, blockchain::PrivateKey, template_kind::TemplateKind};

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
  pub async fn token(&self) -> ConstataResult<String> {
    Ok(self.access_token().await?.attrs.token)
  }                              

  pub async fn valid_until(&self) -> ConstataResult<Option<UtcDateTime>> {
    Ok(self.access_token().await?.attrs.auto_expires_on)
  }

  pub async fn org(&self) -> sqlx::Result<Org> {
    self.document().await?.org().await
  }

  pub async fn entry_optional(&self) -> sqlx::Result<Option<Entry>> {
    self.document().await?.entry_optional().await
  }    

  pub async fn html_proof(&self, key: &PrivateKey, lang: i18n::Lang) -> ConstataResult<String> {
    self.document().await?
      .story().await?
      .proof(self.state.settings.network, &key).await?
      .render_html(lang)
  }

  pub async fn full_url(&self) -> ConstataResult<String> {
    Ok(format!("{}/download-proof/{}", &self.state.settings.url, self.token().await?))
  }

  pub async fn safe_env_url(&self) -> ConstataResult<String> {
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

  pub async fn image_url(&self) -> ConstataResult<String> {
    let image = match self.org().await?.attrs.logo_url {
      Some(url) => url,
      None => self.state.settings.default_logo_url().to_string(),
    };

    Ok(image)
  }

  pub async fn title(&self) -> ConstataResult<Option<String>> {
    if let Some(entry) = self.entry_optional().await? {
     entry.title().await
    } else {
      Ok(None)
    }
  }
  pub async fn template_kind(&self) -> ConstataResult<Option<TemplateKind>> {
    Ok(match self.entry_optional().await? {
      Some(e) => Some(e.template_kind().await?),
      None => None,
    })
  }

  pub async fn share_on_social_networks_call_to_action(&self, l: &i18n::Lang) -> ConstataResult<String> {
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

  pub async fn abridged_pdfs_zip(&self, l: i18n::Lang) -> ConstataResult<(String, Vec<u8>)> {
    use std::io::Write;
    use zip::write::FileOptions;

    let mut destination_buffer = vec![];

    {
      let mut destination = zip::ZipWriter::new(std::io::Cursor::new(&mut destination_buffer));
      destination.start_file("español.pdf", FileOptions::default())?;
      destination.write_all(&AbridgedPdfGenerator::generate(&self, i18n::Lang::Es).await?)?;
      destination.flush()?;
      destination.start_file("english.pdf", FileOptions::default())?;
      destination.write_all(&AbridgedPdfGenerator::generate(&self, i18n::Lang::En).await?)?;
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
}

impl InsertDownloadProofLink {
  pub async fn new(document: &Document, duration_days: i64) -> ConstataResult<Self> {
    let org = document.org().await?;
    let person = org.admin().await?;
    let access_token = org.state.access_token()
      .create(&person, AccessTokenKind::DownloadProofLink, Some(duration_days)).await?;

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

    let template = entry.issuance().await?.template().await?;
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

    std::fs::write("../target/artifacts/test_es.pdf", &AbridgedPdfGenerator::generate(&download_proof_link, i18n::Lang::Es).await?)?;
    std::fs::write("../target/artifacts/test_en.pdf", &AbridgedPdfGenerator::generate(&download_proof_link, i18n::Lang::En).await?)?;
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

    let template = entry.issuance().await?.template().await?;
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
