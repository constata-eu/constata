use serde::Serialize;
use crate::{ Result, Base64Standard};
use super::{
  Site,
  UtcDateTime,
  Decimal,
  PersonId,
  Endorsement,
  document::Document as DbDocument,
  story::Story as DbStory,
  document_part::DocumentPart as DbDocumentPart,
  document_part_signature::DocumentPartSignature as DbDocumentPartSignature,
  bulletin,
};
use std::collections::{HashMap, HashSet};
use serde_with::serde_as;

// Story HTML proofs become temporarily unavailable while new documents are being stamped.

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct StoryBundle {
  pub id: i32,
  pub markers: String,
  pub open_until: Option<UtcDateTime>,
  pub documents: Vec<Document>,
  pub bulletins: HashMap<i32, bulletin::Flow>,
  pub endorsements: HashMap<PersonId, Vec<Endorsement>>,
  pub buy_tokens_link: Option<String>,
}

impl StoryBundle {
  pub async fn from_story(db_story: &DbStory) -> Result<StoryBundle> {
    let mut documents = vec![];
    let mut bulletin_ids: HashSet<i32> = HashSet::new();
    let mut endorsements = HashMap::new();
    
    for db_document in db_story.documents().await?.into_iter() {
      let (document, more_bulletin_ids, person_ids) = Document::from_document(&db_document).await?;
      bulletin_ids = &bulletin_ids | &more_bulletin_ids;

      for i in person_ids {
        endorsements.insert(i, db_story.state.person().find(&i).await?.endorsements().await?);
      }
      
      documents.push(document);
    }

    let account_state = db_story.org().await?.account_state().await?;

    Ok(StoryBundle{
      id: db_story.attrs.id,
      documents,
      markers: db_story.attrs.markers.clone(),
      open_until: db_story.attrs.open_until,
      bulletins: bulletins_from_ids(&db_story.state, bulletin_ids).await?,
      endorsements,
      buy_tokens_link: account_state.pending_invoice_link_url,
    })
  }
}

pub async fn bulletins_from_ids(site: &Site, set: HashSet<i32>) -> Result<HashMap<i32, bulletin::Flow>> {
  let mut bulletins = HashMap::new();
  for id in set {
    bulletins.insert(id, site.bulletin().find(&id).await?.flow());
  }
  Ok(bulletins)
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Document {
  pub id: String,
  pub person_id: PersonId,
  pub author_id: PersonId,
  pub state: String,
  pub created_at: UtcDateTime,
  pub cost: Decimal,
  pub gift_id: Option<i32>,
  pub parts: Vec<DocumentPart>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bulletin_id: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub funded_at: Option<UtcDateTime>,
}

impl Document {
  pub async fn from_document(document: &DbDocument) -> Result<(Document, HashSet<i32>, HashSet<PersonId>)> {
    let mut bulletin_ids = HashSet::new();
    let mut person_ids = HashSet::new();
    let mut parts = vec![];

    person_ids.insert(document.attrs.person_id);

    for part in document.document_part_vec().await?.into_iter() {
      parts.push(DocumentPart::from_part(part).await?);
    }

    if let Some(bulletin_id) = document.attrs.bulletin_id {
      bulletin_ids.insert(bulletin_id);
    }

    for part in &parts {
      for sig in &part.signatures {
        if let Some(bulletin_id) = sig.bulletin_id {
          person_ids.insert(document.state.pubkey().find(&sig.pubkey_id).await?.attrs.person_id);
          bulletin_ids.insert(bulletin_id);
        }
      }
    }

    let state = if document.is_parked() { "Parked" } else { "Accepted" };

    let doc = Document {
      id: document.attrs.id.clone(),
      state: state.to_string(),
      person_id: document.attrs.org_id,
      bulletin_id: document.attrs.bulletin_id,
      funded_at: document.attrs.funded_at,
      created_at: document.attrs.created_at,
      cost: document.attrs.cost,
      author_id: document.attrs.person_id,
      gift_id: document.attrs.gift_id,
      parts,
    };

    Ok((doc, bulletin_ids, person_ids))
  }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct DocumentPart {
  pub id: String,
  pub document_id: String,
  pub friendly_name: String,
  pub hash: String,
  pub content_type: String,
  pub size_in_bytes: i32,
  pub signatures: Vec<Signature>,
  pub is_base: bool
}

impl DocumentPart {
  pub async fn from_part(part: DbDocumentPart) -> Result<DocumentPart> {
    let mut signatures = vec![];

    for sig in part.document_part_signature_vec().await?.into_iter() {
      signatures.push(Signature::from_sig(sig).await?);
    }

    Ok(DocumentPart {
      id: part.attrs.id,
      document_id: part.attrs.document_id,
      friendly_name: part.attrs.friendly_name,
      hash: part.attrs.hash,
      content_type: part.attrs.content_type,
      size_in_bytes: part.attrs.size_in_bytes,
      is_base: part.attrs.is_base,
      signatures,
    })
  }
}

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Signature {
  pub id: i32,
  pub document_part_id: String,
  pub pubkey_id: String,
  pub person_id: i32,
  #[serde(with = "Base64Standard")]
  pub signature: Vec<u8>,
  pub signature_hash: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bulletin_id: Option<i32>,
}

impl Signature {
  pub async fn from_sig(sig: DbDocumentPartSignature) -> Result<Signature> {
    let person_id = sig.pubkey().await?.attrs.person_id;
    Ok(Signature{
      id: sig.attrs.id,
      document_part_id: sig.attrs.document_part_id,
      pubkey_id: sig.attrs.pubkey_id,
      person_id,
      signature: sig.attrs.signature,
      signature_hash: sig.attrs.signature_hash,
      bulletin_id: sig.attrs.bulletin_id,
    })
  }
}
