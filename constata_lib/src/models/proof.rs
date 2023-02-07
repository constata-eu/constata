use crate::{
  signed_payload::SignedPayload,
  models::{
    bulletin::Published,
    story_bundle::*,
    Story,
    PersonId,
    Person,
    Endorsement,
  },
  Base64Standard, Error, Result,
};
use bitcoin::{ PrivateKey, network::constants::Network};
use serde::Serialize;
use serde_with::serde_as;
use std::fs::File;
use std::collections::HashMap;

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Proof<'a> {
  bulletins: Vec<BulletinContents>,
  documents: Vec<DocumentContents>,
  endorsements: HashMap<PersonId, Vec<Endorsement>>,
  persons_missing_kyc: Vec<PersonId>,
  explorers: Vec<String>,
  secure_origin: String,
  will_be_updated: bool,
  #[serde(skip_serializing)]
  key: &'a PrivateKey,
}

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct DocumentContents {
  bulletin_id: i32,
  id: String,
  author_id: PersonId,
  parts: Vec<PartContents>,
}

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PartContents {
  object: DocumentPart,
  #[serde(with = "Base64Standard")]
  contents: Vec<u8>,
}

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct BulletinContents {
  object: Published,
  contents: String,
}

impl<'a> Proof<'a> {
  pub async fn new(story: &Story, network: Network, key: &'a PrivateKey) -> Result<Proof<'a>> {
    use std::cmp::Ordering;

    let bundle = StoryBundle::from_story(&story).await?;

    let mut bulletins = vec![];
    let mut pending_bulletins = vec![];
    for (id, bulletin) in bundle.bulletins.into_iter() {
      if let Ok(object) = bulletin.in_published() {
        let contents = object.payload().await?;
        bulletins.push(BulletinContents { object, contents });
      } else {
        pending_bulletins.push(id);
      }
    }

    let mut documents = vec![];
    let mut parked_documents = vec![];
    for doc in bundle.documents {
      let bulletin_id = if let Some(i) = doc.bulletin_id {
        i
      } else {
        parked_documents.push(doc.id.clone());
        continue;
      };

      if bulletins.iter().find(|b| *b.object.id() == bulletin_id).is_none() {
        continue;
      }

      let mut parts = vec![];
      for object in doc.parts {
        let contents = story.state.document_part().find(&object.id).await?.contents().await?;
        parts.push(PartContents { object, contents });
      }

      let numbered = regex::Regex::new(r"^\d{1,3}[-_].*").unwrap();

      parts.sort_by(|a,b|{
        if a.object.is_base { return Ordering::Less }
        if b.object.is_base { return Ordering::Greater }

        match (numbered.is_match(&a.object.friendly_name), numbered.is_match(&b.object.friendly_name)) {
          (true, false) => Ordering::Less,
          (false, true) => Ordering::Greater,
          (true, true) => a.object.friendly_name.cmp(&b.object.friendly_name),
          (false, false) => Self::type_priority(&a.object.content_type).cmp(&Self::type_priority(&b.object.content_type)),
        }
      });

      documents.push(DocumentContents{
        id: doc.id.clone(),
        author_id: doc.author_id,
        parts,
        bulletin_id
      });
    }

    if documents.is_empty() && !parked_documents.is_empty() {
      return Err(Error::DocumentParked(parked_documents.pop().expect("At least one document pending")));
    }

    if bulletins.is_empty() && !pending_bulletins.is_empty() {
      return Err(Error::WaitForBulletin(pending_bulletins[0]));
    }

    let will_be_updated = !pending_bulletins.is_empty() || !parked_documents.is_empty();

    let secure_origin = story.state.settings.url.clone();

    let explorers = match network {
      Network::Bitcoin => vec![
        "https://blockstream.info/api/tx/".into(),
        "https://btc2.trezor.io/api/v1/tx/".into(),
        "https://mempool.space/api/tx/".into(),
        "https://api.constata.eu/explorer/".into(),
      ],
      Network::Testnet => vec![
        "https://blockstream.info/testnet/api/tx/".into(),
        "https://tbtc1.trezor.io/api/v1/tx/".into(),
        "https://mempool.space/testnet/api/tx/".into(),
        "https://api-staging.constata.eu/explorer/".into(),
      ],
      _ => vec![
        format!("{secure_origin}/explorer/"),
        format!("{secure_origin}/explorer/")
      ],
    };

    let mut persons_missing_kyc = vec![];
    for (person_id, endorsements) in &bundle.endorsements {
      if !endorsements.iter().find(|e| matches!(e, Endorsement::Kyc{..})).is_some() {
        persons_missing_kyc.push(*person_id);
      }
    }

    Ok(Proof {
      bulletins,
      key,
      documents,
      explorers,
      endorsements: bundle.endorsements,
      persons_missing_kyc,
      secure_origin,
      will_be_updated,
    })
  }

  pub fn type_priority(t: &str) -> i32 {
    if t == "application/pdf" {
      1
    } else if t.starts_with("image") {
      2
    } else if t.starts_with("video") {
      3
    } else if t == "text/html" {
      4
    } else if t == "text/plain" {
      5
    } else {
      6
    }
  }

  pub fn render_html(&self, lang: i18n::Lang) -> Result<String> {
    self.render_signed_html(&i18n::Context::from_serialize(&self)?, lang)
  }

  pub fn generate_each_part_html(&self, lang: i18n::Lang) -> Result<Vec<(String, String)>> {
    let mut tuples = vec![];

    for doc in &self.documents {
      for part in &doc.parts {
        let new_doc = DocumentContents{ parts: vec![part.clone()], ..doc.clone()};
        let proof = Proof { documents: vec![new_doc], ..self.clone() };
        let html = self.render_signed_html(&i18n::Context::from_serialize(&proof)?, lang)?;
        let filename = format!("{}_{}.html", &part.object.friendly_name, &part.object.hash);
        tuples.push((html, filename));
      }
    }

    Ok(tuples)
  }

  pub fn render_signed_html(&self, context: &i18n::Context, lang: i18n::Lang) -> Result<String> {
    let mut html = i18n::render(lang, "proofs/html.tera", context)?;
    let signature = SignedPayload::sign_with_key(&html.as_bytes(), &self.key);
    let mut sign_context = i18n::Context::new();
    sign_context.insert("signature", &signature.to_base64());
    html.push_str(&i18n::render(lang, "proofs/signature.tera", &sign_context)?);
    Ok(html)
  }

  pub fn generate_each_part_html_and_zip(&self, lang: i18n::Lang) -> Result<File> {
    use std::io::Write;
    use zip::write::FileOptions;

    let mut file = tempfile::NamedTempFile::new()?;
    {
      let mut zip = zip::ZipWriter::new(&mut file);

      for (html, name) in self.generate_each_part_html(lang)?.iter() {
        zip.start_file(name, FileOptions::default())?;
        zip.write_all(html.as_bytes())?;
        zip.flush()?;
      }
      zip.finish()?;
    }
    Ok(file.reopen()?)
  }

  pub async fn render_endorsements(person: &Person, lang: i18n::Lang) -> Result<String> {
    let mut context = i18n::Context::new();
    context.insert("person_id", person.id());
    context.insert("endorsements", &person.endorsements().await?);
    Ok(i18n::render(lang, "proofs/endorsements.tera", &context)?)
  }
}

describe! {
  use bitcoin::Network;

  dbtest!{ can_generate_endorsement_html(_site, c) 
    let alice = c.enterprise().await;
    alice.make_pubkey_domain_endorsement().await;
    alice.make_kyc_endorsement().await;
    alice.make_email("alice@gmail.com").await;
    assert_that!(
      &Proof::render_endorsements(&alice.person().await, i18n::Lang::Es).await?,
      rematch("residente en España")
    );
    assert_that!(
      &Proof::render_endorsements(&alice.person().await, i18n::Lang::En).await?,
      rematch("lives in España")
    );
  }

  regtest!{ can_generate_document_html_proof(site, c, mut chain)
    use std::io::Read;

    let mut alice = c.enterprise().await;
    alice.make_pubkey_domain_endorsement().await;
    alice.make_email("alice@gmail.com").await;

    let story = alice.story_with_signed_doc(&read("document.zip"), None, "").await;
    let key = TestBlockchain::default_private_key().await?;
    let make_proof = ||{ story.proof(Network::Regtest, &key) };

    assert_that!( &make_proof().await.unwrap_err(), is_variant!{ Error::DocumentParked });

    alice = alice.add_funds().await;

    assert_that!(
      &make_proof().await.unwrap_err(),
      structure!{ Error::WaitForBulletin[ eq(1) ] }
    );

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let proof = make_proof().await?;
    let content = proof.render_html(i18n::Lang::Es).expect("Content to be ready now");

    let bulletin = site.bulletin().find(&1).await?;
    let payload = bulletin.payload().await.unwrap();

    assert_that!(&content, rematch(&bulletin.transaction_hash().as_ref().unwrap()));
    assert_that!(&content, rematch(&payload));
    assert_that!(&content, rematch("</html>"));
    std::fs::write("../target/artifacts/proof_with_kyc_notice.html", &content)?;

    alice.make_kyc_endorsement().await;
    let endorsed_content = proof.render_html(i18n::Lang::Es).expect("Content to be ready now");
    std::fs::write("../target/artifacts/proof.html", &endorsed_content)?;


    assert!(!make_proof().await?.will_be_updated);

    c.bot().await.witnessed_email(&story, samples::multipart_email().as_bytes(), None).await;

    let incomplete = make_proof().await?;
    assert!(incomplete.will_be_updated);
    assert_eq!(incomplete.documents.len(), 1);
    assert_eq!(incomplete.bulletins.len(), 1);
    chain.simulate_stamping().await;

    let complete = make_proof().await?;
    assert!(!complete.will_be_updated);
    assert_eq!(complete.documents.len(), 2);
    assert_eq!(complete.bulletins.len(), 2);

    let content_story = make_proof().await?.render_html(i18n::Lang::Es).expect("Content to be generated");
    assert_that!(&content_story, rematch("Contiene <strong>2 documentos"));

    // And then it can generate a zip file with everything in it
    let mut zipfile = zip::ZipArchive::new(proof.generate_each_part_html_and_zip(i18n::Lang::Es).unwrap()).unwrap();

    let mut inner = zipfile.by_index(0).unwrap();
    assert_eq!(
      inner.name(),
      "full_zip_file_c89934a83069a098fad30ebb3b067f5f7931283ea1d7b6d2a91c3614ce0d3d99.html"
    );
    let mut contents = String::new();
    inner.read_to_string(&mut contents).unwrap();
    assert_that!(&contents, rematch("</html>"));

    assert_eq!(
      Proof::new(&story, Network::Bitcoin, &key).await?.explorers[0],
      "https://blockstream.info/api/tx/"
    );
    assert_eq!(
      Proof::new(&story, Network::Testnet, &key).await?.explorers[0],
      "https://blockstream.info/testnet/api/tx/"
    );
  }
}
