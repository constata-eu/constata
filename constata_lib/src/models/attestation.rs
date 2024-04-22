use crate::{
  ConstataResult,
  signed_payload::SignedPayload,
  models::{
    InsertEmailCallback,
    model,
    Person,
    Story,
    Org,
    OrgDeletion,
    UtcDateTime,
    Site,
  },
};
use juniper::GraphQLObject;

model!{
  state: Site,
  table: attestations,
  struct Attestation {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(text)]
    markers: String,
    #[sqlx_model_hints(int4)]
    story_id: i32,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Org(org_id),
    Person(person_id),
    Story(story_id),
    OrgDeletion(deletion_id),
  }
}

impl AttestationHub {
  pub async fn create(&self,
    person: &Person,
    signed_payloads: &[SignedPayload],
    open_until: Option<UtcDateTime>,
    markers: Option<String>,
    maybe_lang: Option<i18n::Lang>,
    email_admin_access_url_to: Vec<String>,
  ) -> ConstataResult<Attestation> {
    let lang = maybe_lang.unwrap_or(person.attrs.lang);
    let story = self.state.story().create(
      person.attrs.org_id,
      open_until,
      markers.clone().unwrap_or_else(|| String::new()),
      lang
    ).await?;
    let mut documents = vec![];
    for payload in signed_payloads {
      documents.push(self.state.document().create_from_signed_payload(&story, payload, None).await?);
    }

    if let Some(document) = documents.get(0) {
      for address in &email_admin_access_url_to { 
        if person.can_send_email().await? {
          self.state.email_callback().insert(InsertEmailCallback{
            address: address.clone(),
            document_id: document.attrs.id.clone(),
            custom_message: None,
            sent_at: None,
          }).validate_and_save().await?;
        }
      }
    }
    
    Ok(self.insert(InsertAttestation{
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      story_id: story.attrs.id,
      markers: markers.unwrap_or_else(|| String::new()),
    }).save().await?)
  }
}

impl Attestation {
  pub async fn on_done(&self) -> ConstataResult<()> {
    self.state.web_callback().schedule_attestation_done(self).await?;
    Ok(())
  }
}

pub mod for_api {
  use super::*;
  use rust_decimal_macros::dec;
  use num_traits::ToPrimitive;
  use crate::models::endorsement_for_api::EndorsementManifest;

  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  #[graphql(description = "An Attestation over several documents")]
  pub struct Attestation {
    pub id: i32,
    pub person_id: i32,
    pub org_id: i32,
    pub markers: String,
    pub open_until: Option<UtcDateTime>,
    pub state: String,
    pub parking_reason: Option<String>,
    pub done_documents: i32,
    pub parked_documents: i32,
    pub processing_documents: i32,
    pub total_documents: i32,
    pub tokens_cost: f64,
    pub tokens_paid: f64,
    pub tokens_owed: f64,
    pub buy_tokens_url: Option<String>,
    pub accept_tyc_url: Option<String>,
    pub last_doc_date: Option<UtcDateTime>,
    pub email_admin_access_url_to: Vec<String>,
    pub admin_access_url: Option<String>,
    pub public_certificate_url: Option<String>,
    pub created_at: UtcDateTime,
    pub documents: Vec<AttestationDocument>
  }

  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  #[graphql(description = "A document that is part of a whole attestation")]
  pub struct AttestationDocument {
    certification_date: Option<UtcDateTime>,
    parts: Vec<AttestationDocumentPart>
  }

  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  #[graphql(description = "A part of a document. IE: a zip file is part 0, then every file contained is another part.")]
  pub struct AttestationDocumentPart {
    friendly_name: String,
    hash: String,
    is_base: bool,
    signatures: Vec<AttestationSignature>
  }

  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  #[graphql(description = "One of possibly many signatures applied to an attestation document")]
  pub struct AttestationSignature {
    certification_date: Option<UtcDateTime>,
    public_key: String,
    signature: String,
    signature_hash: String,
    endorsement_manifest: EndorsementManifest
  }

  pub async fn from_model(d: super::Attestation) -> ConstataResult<Attestation> {
    let story = d.story().await?;
    let account_state = d.org().await?.account_state().await?;

    let mut email_admin_access_url_to = std::collections::HashSet::new();
    let mut tokens_cost = dec!(0);
    let mut tokens_paid = dec!(0);
    let mut tokens_owed = dec!(0);
    let mut done_documents = 0;
    let mut parked_documents = 0;
    let mut processing_documents = 0;
    let mut documents = vec![];

    for doc in &story.documents().await? {
      tokens_cost += doc.attrs.cost;
      if doc.attrs.funded {
        tokens_paid += doc.attrs.cost;
      } else {
        tokens_owed += doc.attrs.cost;
      }
      let bulletin = doc.bulletin().await?;
      if bulletin.as_ref().map(|b| b.is_published()).unwrap_or(false) {
        done_documents += 1;
      } else if doc.is_parked() {
        parked_documents += 1;
      } else {
        processing_documents += 1;
      }
      for cb in doc.email_callback_scope().all().await? {
        email_admin_access_url_to.insert(cb.attrs.address);
      }

      let mut parts = vec![];
      for part in doc.document_part_vec().await? {
        let mut signatures = vec![];
        for signature in part.document_part_signature_vec().await? {
            let certification_date = signature.bulletin().await?.and_then(|b| b.attrs.block_time);
            let pubkey = signature.pubkey().await?;
            let endorsement_manifest = EndorsementManifest::from_person(&pubkey.person().await?, i18n::Lang::En).await?;
            signatures.push(AttestationSignature{
                certification_date,
                public_key: pubkey.attrs.id,
                signature: base64::encode(signature.attrs.signature),
                signature_hash: signature.attrs.signature_hash,
                endorsement_manifest
            });
        }
        parts.push(AttestationDocumentPart{
            friendly_name: part.attrs.friendly_name,
            hash: part.attrs.hash,
            is_base: part.attrs.is_base,
            signatures
        });
      }

      documents.push(AttestationDocument{
        certification_date: bulletin.and_then(|b| b.attrs.block_time),
        parts
      });
    }

    let state = if done_documents > 0 {
      if parked_documents == 0 && processing_documents == 0 {
        "done"
      } else if parked_documents > 0 {
        "updates_parked"
      } else {
        "updates_processing"
      }
    } else {
      if parked_documents > 0 {
        "parked"
      } else {
        "processing"
      }
    };

    let parking_reason = if state == "parked" || state == "updates_parked" {
      if account_state.pending_tyc_url.is_some() {
        Some("must_accept_tyc")
      } else {
        Some("must_buy_tokens")
      }
    } else {
      None
    };

    let maybe_link = story.get_or_create_download_proof_link(30).await?;
    let (admin_access_url, public_certificate_url) = if let Some(link) = maybe_link {
      (
        Some(link.safe_env_url().await?),
        if link.published_at().is_some() { Some(link.public_certificate_url()) } else { None }
      )
    } else {
      ( None, None )
    };

    let last_doc_date = story.documents().await?.last().map(|d| d.attrs.created_at.clone());

    Ok(Attestation {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      state: state.to_string(),
      parking_reason: parking_reason.map(|x| x.to_string()),
      open_until: story.attrs.open_until,
      markers: d.attrs.markers,
      created_at: d.attrs.created_at,
      email_admin_access_url_to: Vec::from_iter(email_admin_access_url_to),
      admin_access_url,
      public_certificate_url,
      buy_tokens_url: account_state.pending_invoice_link_url,
      accept_tyc_url: account_state.pending_tyc_url,
      done_documents,
      parked_documents,
      processing_documents,
      total_documents: done_documents + parked_documents + processing_documents,
      tokens_cost: tokens_cost.to_f64().unwrap_or(0.0),
      tokens_paid: tokens_paid.to_f64().unwrap_or(0.0),
      tokens_owed: tokens_owed.to_f64().unwrap_or(0.0),
      last_doc_date,
      documents
    })
  }
}

describe! {
  dbtest!{ creates_an_attestation (site, c)
    let alice = c.alice().await;
    let payloads = vec![
      alice.signed_payload(b"hello world"),
      alice.signed_payload(b"goodbye world"),
    ];
    let att = site.attestation().create(&alice.person().await, &payloads, None, None, None, vec![]).await?;
    assert_eq!(att.org_id(), alice.org().await.id());
  }
}
