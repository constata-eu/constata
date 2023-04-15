use crate::{
  models::{
    model,
    hasher::hexdigest,
    Site,
    bulletin,
    PersonId,
    UtcDateTime,
    Decimal,
    Bulletin,
    Person,
    Story,
    Org,
    OrgDeletion,
    DocumentSource,
    document_part::{DocumentPart, SelectDocumentPartHub},
    email_callback::*,
    download_proof_link::*,
    certos::entry::{Entry, SelectEntryHub},
  },
  signed_payload::SignedPayload,
  Error, Result,
};
use rust_decimal_macros::dec;

use mailparse::*;
use serde::Serialize;
use serde_with::serde_as;
use duplicate::duplicate_item;
use chrono::{Utc, Duration};

model!{
  state: Site,
  table: documents,
  #[serde_as]
  struct Document {
    #[sqlx_model_hints(varchar)]
    id: String,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: Option<i32>,
    #[sqlx_model_hints(boolean, default)]
    funded: bool,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(timestamptz, default)]
    funded_at: Option<UtcDateTime>,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(decimal)]
    cost: Decimal,
    #[sqlx_model_hints(decimal)]
    gift_id: Option<i32>,
    #[sqlx_model_hints(int4)]
    story_id: i32,
    #[sqlx_model_hints(varchar, default)]
    #[serde(skip_serializing)]
    delete_parked_token: Option<String>,
    #[sqlx_model_hints(int4, default)]
    #[serde(skip_serializing)]
    deletion_id: Option<i32>,
    #[sqlx_model_hints(document_source)]
    sourced_from: DocumentSource,
  },
  queries {
    all_old_parked("now() - created_at > $1 AND NOT funded AND bulletin_id IS NULL", delete_interval: Duration),
  },
  belongs_to {
    Org(org_id),
    Person(person_id),
    Story(story_id),
    Bulletin(bulletin_id),
    OrgDeletion(deletion_id),
  },
  has_many {
    DocumentPart(document_id),
    EmailCallback(document_id),
    DownloadProofLink(document_id),
    Entry(document_id),
  }
}

pub type MimeOverride = Option<(String, String)>;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  Parked(Parked),
  Accepted(Accepted),
}

#[duplicate_item(flow_variant; [ Parked ]; [ Accepted ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(Document);

#[duplicate_item(flow_variant; [ Parked ]; [ Accepted ];)]
impl flow_variant {
  pub fn into_inner(self) -> Document { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a Document { &self.0 }
}

impl Document {
  pub async fn entry_optional(&self) -> sqlx::Result<Option<Entry>> {
    self.entry_scope().optional().await
  }

  pub fn flow(&self) -> Flow {
    if self.is_accepted() {
      Flow::Accepted(Accepted(self.clone()))
    } else {
      Flow::Parked(Parked(self.clone()))
    }
  }

  pub fn is_accepted(&self) -> bool { *self.funded() }
  pub fn is_parked(&self) -> bool { !*self.funded() }

  #[duplicate_item(
    in_state        state;
    [ in_parked   ] [ Parked    ];
    [ in_accepted ] [ Accepted  ];
  )]
  pub fn in_state(&self) -> Result<state> {
    self.flow().in_state()
  }
}

#[duplicate_item(
  in_state        is_state        variant(i)             state;
  [ in_parked   ] [ is_parked   ] [ Flow::Parked(i)    ] [ Parked    ];
  [ in_accepted ] [ is_accepted ] [ Flow::Accepted(i)  ] [ Accepted  ];
)]
impl Flow {
  pub fn in_state(self) -> Result<state> {
    if let variant([inner]) = self {
      Ok(inner)
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a Document {
    match self {
      Flow::Accepted(a) => a.as_inner(),
      Flow::Parked(a) => a.as_inner(),
    }
  }
}

impl Parked {
  pub async fn delete_parked(&self) -> Result<()> {
    if !self.0.reloaded().await?.can_be_deleted() {
      return Err(Error::validation("document/accepted", "this_documents_is_not_parked"));
    }
    self.as_inner().state.db.execute(sqlx::query!(
      "DELETE FROM documents WHERE id = $1 AND bulletin_id IS NULL AND NOT FUNDED",
      &self.0.attrs.id
    )).await?;

    Ok(())
  }
}

impl Accepted {
  pub async fn bulletin(&self) -> sqlx::Result<Bulletin> {
    Ok(self.0.bulletin().await?.expect("Accepted to have a bulletin"))
  }

  pub fn bulletin_id(&self) -> i32 {
    self.0.attrs.bulletin_id.expect("Accepted to have a bulletin")
  }
}

impl Document {
  pub async fn base_document_part(&self) -> sqlx::Result<DocumentPart> {
    self.document_part_scope().is_base_eq(true).one().await
  }

  pub async fn attachments_count(&self) -> Result<i32> {
    Ok(self.document_part_scope().count().await? as i32)
  }

  pub fn can_be_deleted(&self) -> bool {
    if self.bulletin_id().is_none() && self.attrs.funded == false && self.is_parked() {
      return true;
    } else {
      return false;
    }
  }

  pub fn delete_parked_url(&self) -> String {
    format!(
      "{}/delete_parked/{}",
      &self.state.settings.url,
      self.attrs.delete_parked_token.clone().unwrap_or("".to_string())
    )
  }

  pub async fn eta(&self) -> Option<i64> {
    match self.bulletin().await.ok()? {
      None => None,
      Some(b) => {
        match b.flow() {
          bulletin::Flow::Draft(bulletin) => {
            let bulletin_life = Utc::now().signed_duration_since(*bulletin.started_at()).num_minutes();
            Some((60 - bulletin_life).max(0) + 20)
          },
          bulletin::Flow::Proposed(_) | bulletin::Flow::Submitted(_) => Some(20),
          bulletin::Flow::Published(_) => None,
        }
      }
    }
  }

  pub async fn size_in_megabytes(&self) -> Result<f64> {
    let size = self.base_document_part().await?.attrs.size_in_bytes;
    Ok((size as f64) / 1024_f64 / 1024_f64)
  }

  pub async fn friendly_name(&self) -> Result<String> {
    Ok(self.base_document_part().await?.attrs.friendly_name)
  }

  pub async fn create_parts(&self, payload: &[u8], filename: Option<&str>, mime: MimeOverride) -> Result<()> {
    let mime_and_ext = mime.unwrap_or_else(|| Self::mime_and_ext(payload, filename));
    match (mime_and_ext.0.as_str(), mime_and_ext.1.as_str()) {
      ("message/rfc822", _) => self.index_as_email(payload).await,
      ("application/zip", _) => self.index_as_zip(payload, true).await,
      (media_type, ext) => self.index_as_file(payload, media_type, &ext).await,
    }
  }

  async fn index_as_file(&self, payload: &[u8], media_type: &str, ext: &str) -> Result<()> {
    self.state.document_part().create(
      true,
      self.id(),
      &format!("document{}", ext),
      media_type,
      payload,
    )
    .await?;
    Ok(())
  }

  async fn index_as_email(&self, main_payload: &[u8]) -> Result<()> {
    let parsed = parse_mail(&main_payload)?;

    if parsed.subparts.is_empty() && parsed.get_body_raw()?.is_empty() {
      return Err(Error::validation("empty_email", "email was empty"));
    }

    let email_name = parsed
      .headers
      .get_first_value("Subject")
      .unwrap_or_else(|| "full_email_message".to_string() );

    self.state.document_part().create(
      true,
      self.id(),
      &email_name,
      "message/rfc822",
      &main_payload,
    ).await?;

    let body = parsed.get_body_raw()?;
    if body.len() > 0 {
      self.state.document_part().create(
        false,
        self.id(),
        &format!("{email_name}'"),
        &parsed.ctype.mimetype,
        &body,
      ).await?;
    }

    self.index_email_subparts(&parsed.subparts).await?;

    Ok(())
  }

  #[async_recursion::async_recursion]
  async fn index_email_subparts(&self, subparts: &[mailparse::ParsedMail<'_>]) -> Result<()> {
    for part in subparts {
      let payload = part.get_body_raw()?;
      let (_, ext) = Self::mime_and_ext(&payload, None);

      if payload.len() > 0 {
        let friendly_name = part
          .headers
          .get_first_value("Content-Disposition")
          .and_then(|value| {
            let mut params = mailparse::parse_content_disposition(&value).params;
            params.remove("filename").or_else(|| params.remove("name"))
          })
          .unwrap_or_else(|| format!("unnamed_attachment{}", &ext));

        self.state.document_part().create(
          false,
          self.id(),
          &friendly_name,
          &part.ctype.mimetype,
          &payload,
        ).await?;

        if &part.ctype.mimetype == "application/zip" {
          self.index_as_zip(&payload, false).await?;
        }
      }

      if part.subparts.len() > 0 {
        self.index_email_subparts(&part.subparts).await?;
      }
    }
    Ok(())
  }

  async fn index_as_zip(&self, payload: &[u8], is_base: bool) -> Result<()> {
    use std::io::Read;

    let cursor = std::io::Cursor::new(payload);
    let mut archive = zip::ZipArchive::new(cursor)?;

    if is_base {
      self.state.document_part().create(
        true,
        self.id(),
        "full_zip_file",
        "application/zip",
        payload,
      ).await?;
    }

    for i in 0..archive.len() {
      let (friendly_name, bytes) = {
        let mut file = archive.by_index(i)?;
        if !file.is_file() {
          continue;
        }

        let mut buffer = vec![];
        if file.read_to_end(&mut buffer).is_err() {
          continue;
        }

        match file.enclosed_name() {
          Some(name) => (name.to_string_lossy().to_string(), buffer),
          None => {
            return Err(Error::validation(
              "payload",
              &format!("file {} was malformed", i),
            ))
          }
        }
      };

      let (mime, _) = Self::mime_and_ext(&bytes, Some(&friendly_name));

      self.state.document_part().create(
        false,
        &self.id(),
        &friendly_name,
        &mime,
        &bytes,
      ).await?;
    }

    Ok(())
  }

  pub fn mime_and_ext(bytes: &[u8], filename: Option<&str>) -> (String, String) {
    let mime = tree_magic_mini::from_u8(bytes);

    if let Some(name) = filename {
      let extension = name.split(".").collect::<Vec<&str>>().pop().unwrap_or("");
      match (mime, extension) {
        ("text/plain", "json") => return (format!("application/json"), format!(".json")),
        ("application/zip", "docx") => return ("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(), format!(".docx")),
        ("application/zip", "xlsx") => return ("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(), format!(".xlsx")),
        ("application/zip", "pptx") => return ("application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(), format!(".pptx")),
        (_,_) => (),
      };
    };

    let ext = mime2ext::mime2ext(mime)
      .map(|e| format!(".{}", e))
      .unwrap_or("".to_string());
    (mime.to_string(), ext)
  }

  async fn create_download_proof_link(&self, duration_days: i64) -> Result<DownloadProofLink> {
    Ok(self.state.download_proof_link()
      .insert(InsertDownloadProofLink::new(&self, duration_days).await?)
      .save().await?
    )
  }

  pub async fn get_or_create_download_proof_link(&self, duration_days: i64) -> Result<DownloadProofLink> {
    match self.active_download_proof_link().await? {
      Some(x) => Ok(x),
      _ => self.create_download_proof_link(duration_days).await
    }
  }

  pub async fn active_download_proof_link(&self) -> sqlx::Result<Option<DownloadProofLink>> {
    Ok(self.state.download_proof_link()
      .active_by_document_id(self.id().clone())
      .optional().await?)
  }
}

impl DocumentHub {
  pub async fn create_from_signed_payload(&self, story: &Story, signed_payload: &SignedPayload, filename: Option<&str>)
   -> Result<Document> {
    if !signed_payload.signed_ok()? {
      return Err(Error::validation("signed_payload", "wrong_signature"));
    }

    let person_id = self.state.pubkey().find(&signed_payload.signer.to_string())
      .await
      .map_err(|_| Error::validation("signed_payload/signer", "signer_is_unknown"))?
      .attrs
      .person_id;

    let doc = self.state.document().create_and_index(&story, &signed_payload.payload, filename, person_id, None, DocumentSource::Api, false).await?;
    doc.base_document_part().await?.add_signature(&signed_payload).await?;

    Ok(doc)
  }

  pub async fn create_and_index
  (&self, story: &Story, payload: &[u8], filename: Option<&str>, person_id: PersonId, mime_override: MimeOverride, sourced_from: DocumentSource, always_gift: bool)
   -> Result<Document> {
    let org = story.org().await?;
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_nanos();
    let id = format!("{}-{}-{:x}", org.id(), hexdigest(payload), time);

    if self.state.document().select().id_eq(&id).optional().await?.is_some() {
      return Err(Error::validation("uniqueness", "id already exists"));
    }

    let cost = (Decimal::from(payload.len()) / dec!(1024) / dec!(1024)).ceil();
    let gift_id = if always_gift {
      Some(self.state.gift().build(*org.id(), cost, "For internal document").save().await?.attrs.id)
    } else {
      if let Some(s) = org.subscription().await? {
        s.claim_monthly_gift(cost).await?.map(|x| x.attrs.id)
      } else {
        None
      }
    };

    let doc = self.insert(InsertDocument{
      id,
      org_id: *org.id(),
      cost,
      gift_id,
      story_id: *story.id(),
      person_id: person_id,
      sourced_from,
    }).save().await?;

    doc.create_parts(&payload, filename, mime_override).await?;

    story.get_or_create_snapshot().await?;

    org.account_state().await?.fund_all_documents().await?;

    Ok(doc.reloaded().await?)
  }

  pub async fn delete_old_parked(&self) -> Result<()> {
    let delete_interval = self.state.settings.delete_old_parked_interval();
    for doc in self.state.document().all_old_parked(delete_interval).all().await? {
      doc.in_parked()?.delete_parked().await?;
    };
    Ok(())
  }
}

describe! {
  use rust_decimal_macros::dec;
  use crate::models::{document_part_signature::*, document_part::*, };

  dbtest!{ documents_may_be_funded_from_subscription_gifts (_site, c)
    c.alice().await.accepted_document(&samples::multipart_email().as_bytes()).await;
  }

  dbtest!{ documents_are_parked_when_user_has_not_enough_tokens (site, c)
    let user = c.enterprise().await;
    let unfunded_document = user.signed_document(b"hello world").await;
    assert!(unfunded_document.is_parked());
    assert!(unfunded_document.in_parked().is_ok());
    assert!(!unfunded_document.is_accepted());
    assert!(unfunded_document.in_accepted().is_err());
    assert!(unfunded_document.reloaded().await?.is_parked());

    assert_bulletin_payload(site.bulletin().current().await?.as_inner(), 4, vec![
      "347fdfa2e3ac333b2d36a703c6906a16899b9c9abe583d730ea884006abfd525",
      "7212717881098d39375358670512fda7c1c2e3a7f4ac1669933676cc11b21392",
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ]).await;

    user.add_funds().await;
    let funded_document = unfunded_document.reloaded().await?;
    assert!(funded_document.is_accepted());
    assert!(funded_document.in_accepted().is_ok());
    assert!(!funded_document.is_parked());
    assert!(funded_document.in_parked().is_err());

    assert_bulletin_payload(site.bulletin().current().await?.as_inner(), 6, vec![
      "347fdfa2e3ac333b2d36a703c6906a16899b9c9abe583d730ea884006abfd525",
      "3eb9c5d0eea7d812c5dec9e068381170f8e090dc3a79bc655f6a7e21a0e6c80a",
      "7212717881098d39375358670512fda7c1c2e3a7f4ac1669933676cc11b21392",
      "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ]).await;
  }

  dbtest!{ documents_may_be_a_gift_outside_subscriptions (site, c)
    let user = c.enterprise().await;
    assert!(user.signed_document(b"hello world").await.is_parked());

    let doc = site.document().create_and_index(
      &user.make_story().await,
      b"hello world two",
      None,
      user.person_id(),
      None,
      DocumentSource::Internal,
      true
    ).await?;

    assert!(doc.is_accepted());
  }

  dbtest!{ calculates_document_cost (_site, c)
    let u = c.alice().await;
    assert_that!(&u.accepted_document(samples::multipart_email().as_bytes()).await.into_inner().attrs.cost, eq(dec!(1)));
    assert_that!(&u.accepted_document(&vec![1u8; 1024 * 1024 * 5]).await.into_inner().attrs.cost, eq(dec!(5)));
  }

  dbtest!{ creates_a_document_from_an_email (_site, c)
    let document = c.alice().await
      .accepted_document(&samples::multipart_email().as_bytes()).await
      .into_inner();

    assert_that!(&document.attrs, structure![DocumentAttrs {
      cost: eq(dec!(1)),
      id: rematch("1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5-[a-f0-9]{16}"),
      person_id: eq(1),
    }]);

    let parts = document.document_part_vec().await?;

    assert_eq!(parts.len(), 6);

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "holis 😅",
      "997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "message/rfc822",
      1410
    );

    let signatures = parts[0].document_part_signature_vec().await?;

    assert_document_part_signature!(
      &signatures[0].attrs,
      1,
      &parts[0].attrs.id,
      "mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx",
      "1f5c1e75bc8a8bc042b8de10519043be2137794e4143132b34a4ef18091e8848351eb73c96f72b3f6edb75e25f2e2043666ec31e3e4203001432c47b6bab74a0f2",
      "f5e017b6eea9816e33ecc5b13c22479d540973005d47a0f65c9b8db9301d1343",
      Some(1)
    );

    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "hello.txt",
      "ca968e07a1aa7bf2e9e56f7c3414972dc5be236e7e90322be7a3c5f3ffb7b290",
      "text/plain",
      61
    );

    assert_document_part!(
      &parts[2],
      0,
      false,
      "1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "unnamed_attachment.txt",
      "6f17763b3d41b05819a091e5bb972ca08f49e9bfda2dbc33cbd48f2ba289f00d",
      "text/html",
      94
    );

    assert_document_part!(
      &parts[3],
      0,
      false,
      "1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "unnamed_attachment.zip",
      "833530be2b1160db6e5e53c34f128cd86f21197ff1cc58439b2fece49c102c44",
      "application/zip",
      530
    );

    assert_document_part!(
      &parts[4],
      0,
      false,
      "1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "bar/baz.txt",
      "bf07a7fbb825fc0aae7bf4a1177b2b31fcf8a3feeaf7092761e18c859ee52a9c",
      "text/plain",
      4
    );

    assert_document_part!(
      &parts[5],
      0,
      false,
      "1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5",
      "foo.txt",
      "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
      "text/plain",
      4
    );
  }

  dbtest!{ creates_a_document_from_a_single_file (_site, c)
    let document = c.alice().await
      .accepted_document(&read("bitcoin.pdf"))
      .await
      .into_inner();

    assert_eq!(document.cost(), &dec!(1));
    let parts = document.document_part_vec().await?;
    assert_eq!(parts.len(), 1);

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-b1674191a88ec5cdd733e4240a81803105dc412d6c6708d53ab94fc248f4f553",
      "document.pdf",
      "b1674191a88ec5cdd733e4240a81803105dc412d6c6708d53ab94fc248f4f553",
      "application/pdf",
      184292
    );

    let signatures = parts[0].document_part_signature_vec().await?;

    assert_document_part_signature!(
      &signatures[0].attrs,
      1,
      &parts[0].attrs.id,
      "mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx",
      "20c1f46ec0cc9246b1a057577c7ea7cc65f5e63487ac8efd2b6d909c4fb35e509277929ef985a46bac59328733dc298378974601fd2d327d578a707ad82f84b93f",
      "5f2d3c137d3d29bdfbf5f9c76d10848b7ca2a15be0d48045b0f7f3fa4e602860",
      Some(1)
    );
  }

  dbtest!{ creates_a_document_from_a_zip_file (_site, c)
    let document = c.alice().await
      .accepted_document(&read("document.zip"))
      .await
      .into_inner();

    assert_eq!(document.cost(), &dec!(1));

    let parts = document.document_part_vec().await?;
    assert_eq!(parts.len(), 4);

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-c89934a83069a098fad30ebb3b067f5f7931283ea1d7b6d2a91c3614ce0d3d99",
      "full_zip_file",
      "c89934a83069a098fad30ebb3b067f5f7931283ea1d7b6d2a91c3614ce0d3d99",
      "application/zip",
      47012
    );

    let signatures = parts[0].document_part_signature_vec().await?;

    assert_document_part_signature!(
      &signatures[0].attrs,
      1,
      &parts[0].attrs.id,
      "mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx",
      "200840b4a41a70761b242980c16564735d237e6455df4eab38cc9a644a4af28e12128e1af2a95fcf3d01b16e46dfd9c01c82ea788df9e6ebf15a46e3aa88ceac93",
      "d11b07816b41fc640a3ce3d14f0c05f9ee23c739466ca6148c9672996e8a2285",
      Some(1)
    );
    
    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-c89934a83069a098fad30ebb3b067f5f7931283ea1d7b6d2a91c3614ce0d3d99",
      "extras/photo.jpg",
      "9f167c730f2d9eac8c187c6b2654b1860a4e4719b9f35916857e937acc25ea46",
      "image/jpeg",
      46258
    );
    assert!(parts[1].document_part_signature_vec().await?.is_empty());

    assert_document_part!(
      &parts[2],
      0,
      false,
      "1-c89934a83069a098fad30ebb3b067f5f7931283ea1d7b6d2a91c3614ce0d3d99",
      "extras/testfile.txt",
      "03ba204e50d126e4674c005e04d82e84c21366780af1f43bd54a37816b6ab340",
      "text/plain",
      13
    );
    assert!(parts[2].document_part_signature_vec().await?.is_empty());

    assert_document_part!(
      &parts[3],
      0,
      false,
      "1-c89934a83069a098fad30ebb3b067f5f7931283ea1d7b6d2a91c3614ce0d3d99",
      "testfile.txt",
      "03ba204e50d126e4674c005e04d82e84c21366780af1f43bd54a37816b6ab340",
      "text/plain",
      13
    );
    assert!(parts[3].document_part_signature_vec().await?.is_empty());
  }

  dbtest!{ can_get_the_part_content (_site, c)
    let content = c.alice().await
      .accepted_document(b"Hello World!").await
      .into_inner()
      .document_part_vec().await?[0]
      .contents().await?;
    assert_eq!(content, b"Hello World!");
  }

  regtest!{ document_has_an_estimated_time_of_arrival (_site, c, mut chain)
    let user = c.enterprise().await;
    let unfunded_document = user.signed_document(b"hello world").await;
    assert!(unfunded_document.eta().await.is_none());

    user.add_funds().await;

    let funded_document = unfunded_document.reloaded().await?;

    assert!(funded_document.is_accepted());
    assert_eq!( funded_document.eta().await.expect("eta"), 80);

    chain.fund_signer_wallet();
    chain.blockchain.process().await?;
    assert_eq!( funded_document.eta().await.expect("eta"), 20);

    chain.blockchain.process().await?;
    assert_eq!( funded_document.eta().await.expect("eta"), 20);
  }

  dbtest!{ fails_to_create_from_signed_payload_if_signer_is_unknown (site, c)
    let story = c.alice().await.make_story().await;
    let eve = c.eve().await;

    let error = site.document().create_from_signed_payload(
      &story,
      &eve.signed_payload(b"Hello Everyone"),
      None,
    ).await;

    assert_that!(
      &error.unwrap_err(),
      structure!{ Error::Validation{ message: eq("signer_is_unknown".to_string()) } }
    )
  }

  dbtest!{ fails_to_create_from_signed_payload_if_signature_is_wrong (site, c)
    let alice = c.alice().await;
    let story = alice.make_story().await;
    let payload = alice.wrong_signed_payload(&b"Hello Everyone"[..]);
    assert_that!(
      &site.document().create_from_signed_payload(&story, &payload, None).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("wrong_signature".to_string()) } }
    )
  }

  dbtest!{ two_people_may_save_the_same_document (_site, c)
    let one = c.alice().await.signed_document(b"Hello World!").await;
    let two = c.bob().await.signed_document(b"Hello World!").await;

    assert_that!(one.id(), rematch("1-7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069-[a-f0-9]{16}"));
    assert_that!(two.id(), rematch("2-7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069-[a-f0-9]{16}"));
    assert_eq!(one.document_part_vec().await?[0].hash(), two.document_part_vec().await?[0].hash());
  }

  dbtest!{ creates_a_document_from_html_file (_site, c)
    let document = c.alice().await
      .accepted_document_with_filename(
        &read("html_for_testing.html"),
        Some("html_for_testing.html")
      ).await
      .into_inner();
    let parts = document.document_part_vec().await?;

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-75724e3bf619ddf9cc4ab38920592c61b6363c0e38f3465044f18376e604152d",
      "document.html",
      "75724e3bf619ddf9cc4ab38920592c61b6363c0e38f3465044f18376e604152d",
      "text/html",
      682
    );
  }

  dbtest!{ creates_a_document_from_json_file (_site, c)
    let document = c.alice().await
      .accepted_document_with_filename(
        &read("json_for_testing.json"),
        Some("json_for_testing.json")
      ).await
      .into_inner();
    let parts = document.document_part_vec().await?;

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-70c43c342deba0a92bdc52b3455feb65744ceb078972df712216c9667dea9a89",
      "document.json",
      "70c43c342deba0a92bdc52b3455feb65744ceb078972df712216c9667dea9a89",
      "application/json",
      3490
    );
  }

  dbtest!{ creates_a_document_from_docx_file (_site, c)
    let document = c.alice().await
      .accepted_document_with_filename(
        &read("docx_for_testing.docx"),
        Some("docx_for_testing.docx")
      ).await
      .into_inner();
    let parts = document.document_part_vec().await?;

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-a8ae181df5e3a96e1a0a13addd64a1201e4a46c7f500889badc5385ebf0aaacf",
      "document.docx",
      "a8ae181df5e3a96e1a0a13addd64a1201e4a46c7f500889badc5385ebf0aaacf",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      6623
    );
  }

  dbtest!{ creates_a_document_from_xlsx_file (_site, c)
    let document = c.alice().await
      .accepted_document_with_filename(
        &read("xlsx_for_testing.xlsx"),
        Some("xlsx_for_testing.xlsx")
      ).await
      .into_inner();
    let parts = document.document_part_vec().await?;

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-1cba77d345da918e3e0e10e99c911f6f99bd5fce1898e43d3c2cbda8cb44b131",
      "document.xlsx",
      "1cba77d345da918e3e0e10e99c911f6f99bd5fce1898e43d3c2cbda8cb44b131",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      4710
    );
  }

  dbtest!{ creates_a_document_from_pptx_file (_site, c)
    let document = c.alice().await
      .accepted_document_with_filename(
        &read("pptx_for_testing.pptx"),
        Some("pptx_for_testing.pptx")
      ).await
      .into_inner();
    let parts = document.document_part_vec().await?;

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-bacfbbeac447e2be685fd27724fb04ee62a76d6f79b51c16e2285784d1b4b787",
      "document.pptx",
      "bacfbbeac447e2be685fd27724fb04ee62a76d6f79b51c16e2285784d1b4b787",
      "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      32085
    );
  }

  dbtest!{ creates_from_a_zip_file_with_html_json_and_docx (_site, c)
    let document = c.alice().await
      .accepted_document_with_filename(
        &read("html_json_docx_xlsx_pptx.zip"),
        Some("document.zip")
      ).await.into_inner();

    let parts = document.document_part_vec().await?;
    assert_eq!(parts.len(), 6);

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "full_zip_file",
      "90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "application/zip",
      37688
    );
    
    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "docx_for_testing.docx",
      "a8ae181df5e3a96e1a0a13addd64a1201e4a46c7f500889badc5385ebf0aaacf",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      6623
    );

    assert_document_part!(
      &parts[2],
      0,
      false,
      "1-90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "html_for_testing.html",
      "75724e3bf619ddf9cc4ab38920592c61b6363c0e38f3465044f18376e604152d",
      "text/html",
      682
    );

    assert_document_part!(
      &parts[3],
      0,
      false,
      "1-90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "json_for_testing.json",
      "70c43c342deba0a92bdc52b3455feb65744ceb078972df712216c9667dea9a89",
      "application/json",
      3490
    );

    assert_document_part!(
      &parts[4],
      0,
      false,
      "1-90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "pptx_for_testing.pptx",
      "bacfbbeac447e2be685fd27724fb04ee62a76d6f79b51c16e2285784d1b4b787",
      "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      32085
    );

    assert_document_part!(
      &parts[5],
      0,
      false,
      "1-90181496f2346d6362f8b068d3301d21883cf044bac9475472968010273023ad",
      "xlsx_for_testing.xlsx",
      "1cba77d345da918e3e0e10e99c911f6f99bd5fce1898e43d3c2cbda8cb44b131",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      4710
    );
  }

  dbtest!{ creates_json_docx_xlsx_pptx_from_an_email (_site, c)
    let document = c.alice().await
      .accepted_document(&samples::json_docx_and_xlsx_email().as_bytes()).await
      .into_inner();

    assert_that!(&document.attrs, structure!(DocumentAttrs {
      cost: eq(dec!(1)),
      id: rematch("1-f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee-[a-f0-9]{16}"),
      person_id: eq(1),
    }));

    let parts = document.document_part_vec().await?;

    assert_eq!(parts.len(), 5);

    assert_document_part!(
      &parts[0],
      1,
      true,
      "1-f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee",
      "holis 😅",
      "f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee",
      "message/rfc822",
      1950
    );

    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee",
      "json_for_testing.json",
      "c7d67faf3cddaa0071b1af21dfadacc71765236bd2426eb8c12436abd5509594",
      "application/json",
      867
    );

    assert_document_part!(
      &parts[2],
      0,
      false,
      "1-f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee",
      "docx_for_testing.docx",
      "b5c00fb6472add2ea43c05ebcecaa5031ddad23cf82059adad16f2ecf7c41f6a",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      26
    );

    assert_document_part!(
      &parts[3],
      0,
      false,
      "1-f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee",
      "xlsx_for_testing.xlsx",
      "cbfc199c926d485df84824affb865a49212f74c383996e861daff3b32803b175",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      26
    );

    assert_document_part!(
      &parts[4],
      0,
      false,
      "1-f266beb04cb0d438ee2810d0996133c782ea8fd8a3c4203a7c63c5986ac634ee",
      "pptx_for_testing.pptx",
      "5b82ae0a78d1693d9be9cff8c5897b7f13fa5b06a0bc976168de3448ee576057",
      "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      26
    );
  }

  dbtest!{ can_store_documents_remotely (_site, c)
    requires_setting!(storage.key);

    let mut alice = c.alice().await;
    let bucket = alice.db.site.storage.bucket.clone();

    clear_bucket(&bucket).await;

    alice.accepted_document(b"hello world").await.into_inner();
    assert!(list_bucket(&bucket).await.is_empty());

    alice.db.site.storage.local = false;
    let doc = alice.accepted_document(&samples::multipart_email().as_bytes()).await
      .into_inner();

    let part_ids: Vec<String> = doc.document_part_vec().await?
      .into_iter()
      .map(|p| format!("dp-{}", p.attrs.id))
      .collect();

    assert_that!(&list_bucket(&bucket).await, contains_in_any_order(part_ids));

    let parts = doc.document_part_vec().await?;
    assert_eq!(parts.len(), 6);
    for part in parts {
      assert_eq!(hexdigest(&part.contents().await?), part.attrs.hash);
    }
  }

  dbtest!{ can_encrypt_stored_documents (_site, c)
    requires_setting!(storage.key);

    let mut alice = c.alice().await;
    let bucket = alice.db.site.storage.bucket.clone();

    clear_bucket(&bucket).await;
    alice.db.site.storage.files_key = Some("our 32 character long passphrase".to_string());

    let part = alice.accepted_document(b"hello world").await.into_inner().document_part_vec().await?.pop().unwrap();
    assert_eq!(hexdigest(&part.contents().await?), part.attrs.hash);

    alice.db.site.storage.files_key = None;
    let encrypted_hash = hexdigest(&alice.db.site.storage.get(&format!("dp-{}", part.attrs.id)).await?); 
    assert!(encrypted_hash != part.attrs.hash);
  }

  dbtest!{ can_delete_parked_document (_site, c)
    let user = c.enterprise().await;
    let document = user.signed_document(b"hello world").await;

    assert!(document.is_parked());

    let parts = document.document_part_vec().await?;
    let mut signatures = vec![];
    for part in parts.iter() {
      signatures.append(&mut part.document_part_signature_vec().await?)
    }

    assert_that!(document.in_parked()?.delete_parked().await.is_ok());
    assert_that!(document.state.document().find(document.id()).await.is_err());
    assert_that!(document.reloaded().await.is_err());
    for part in parts {
      assert_that!(part.reloaded().await.is_err());
    }
    for signature in signatures {
      assert_that!(signature.reloaded().await.is_err());
    }

    assert_that!(document.reloaded().await.is_err());
  }

  dbtest!{ cannot_delete_accepted_document (_site, c)
    let user = c.enterprise().await;
    let parked_document = user.signed_document(b"hello world").await.in_parked()?.clone();
    assert!(parked_document.0.is_parked());

    user.add_funds().await;

    let accepted_document = parked_document.0.reloaded().await?;
    assert!(accepted_document.is_accepted());
    assert!(parked_document.0.is_parked());

    assert_that!(parked_document.delete_parked().await.is_err());
    assert_that!(parked_document.0.reloaded().await?.is_accepted());
  }

  dbtest!{ delete_old_parked_after_some_time (site, c)
    let enterprise = c.enterprise().await;
    let alice = c.alice().await;

    let sixty_days_parked = create_parked(&site, &enterprise, 60).await?;
    let forthy_days_parked = create_parked(&site, &enterprise, 40).await?;
    let forthy_days_accepted = create_accepted(&alice, &enterprise, 40).await?;
    let thirthy_days_parked = create_parked(&site, &enterprise, 30).await?;
    let twenty_days_parked = create_parked(&site, &enterprise, 20).await?;
    let zero_days_parked = create_parked(&site, &enterprise, 0).await?;
    site.document().delete_old_parked().await?;

    assert_document_existance_is(true, zero_days_parked).await?;
    assert_document_existance_is(true, twenty_days_parked).await?;
    assert_document_existance_is(true, thirthy_days_parked).await?;
    assert_document_existance_is(true, forthy_days_accepted).await?;
    assert_document_existance_is(false, forthy_days_parked).await?;
    assert_document_existance_is(false, sixty_days_parked).await?;
  }

  async fn assert_document_existance_is(should_exist: bool, documents: Vec<Document>) -> Result<()> {
    for document in documents {
      assert!(document.reloaded().await.is_ok() == should_exist)
    }
    Ok(())
  }
  async fn create_parked(site: &Site, enterprise: &SignerClient, days_ago: i64) -> Result<Vec<Document>> {
    let update_created_at = Utc::now() - Duration::days(days_ago);
    let mut parkeds = vec![];
    let counter = site.document().select().person_id_eq(enterprise.person_id.unwrap()).count().await?;
    parkeds.push(enterprise.signed_document(format!("hello world {}", counter).as_bytes()).await);
    parkeds.push(enterprise.signed_document(format!("hello world {}", counter + 1).as_bytes()).await);
    parkeds.push(enterprise.signed_document(format!("hello world {}", counter + 2).as_bytes()).await);
    for parked in parkeds.iter() {
      parked.clone().update().created_at(update_created_at).save().await?;
    }
    Ok(parkeds)
  }
  async fn create_accepted(alice: &SignerClient, enterprise: &SignerClient, days_ago: i64) -> Result<Vec<Document>> {
    let update_created_at = Utc::now() - Duration::days(days_ago);
    let accepted = alice.signed_document(b"hello world").await
      .update().created_at(update_created_at).save().await?;
    let only_funded = enterprise.signed_document(b"only funded").await
      .update().created_at(update_created_at).funded(true).save().await?;
    let only_bulletin_id = enterprise.signed_document(b"only bulletin id").await
      .update().created_at(update_created_at).bulletin_id(Some(1)).save().await?;

    Ok(vec![accepted, only_funded, only_bulletin_id])
  }

  async fn list_bucket(bucket: &s3::bucket::Bucket) -> Vec<String> {
    bucket.list("".to_string(), None).await.unwrap()
      .pop().unwrap()
      .contents.into_iter().map(|o| o.key)
      .collect()
  }

  async fn clear_bucket(bucket: &s3::bucket::Bucket) {
    for name in list_bucket(&bucket).await {
      bucket.delete_object(name).await.unwrap();
    }
    assert!(list_bucket(bucket).await.is_empty());
  }
}
