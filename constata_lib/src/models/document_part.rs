use crate::{
  prelude::*,
  models::{
    model,
    hasher::hexdigest,
    Document,
    document_part_signature::*,
    storable::*,
  },
  signed_payload::SignedPayload,
};

model!{
  state: Site,
  table: document_parts,
  struct DocumentPart {
    #[sqlx_model_hints(varchar)]
    id: String,
    #[sqlx_model_hints(varchar)]
    document_id: String,
    friendly_name: String,
    #[sqlx_model_hints(varchar)]
    hash: String,
    #[sqlx_model_hints(varchar)]
    content_type: String,
    size_in_bytes: i32,
    #[sqlx_model_hints(boolean)]
    is_base: bool
  },
  belongs_to {
    Document(document_id),
  },
  has_many {
    DocumentPartSignature(document_part_id),
  }
}

derive_storable!(DocumentPart, "dp");

impl DocumentPartHub {
  pub async fn create(&self,
    is_base: bool,
    document_id: &str,
    friendly_name: &str,
    content_type: &str,
    payload: &[u8],
  ) -> ConstataResult<DocumentPart> {
    let hash = hexdigest(payload);
    let size_in_bytes = payload.len() as i32;
    let id = hexdigest(
      format!(
        "{}{}{}{}{}",
        &document_id, &hash, &friendly_name, &content_type, &size_in_bytes
      )
      .as_bytes(),
    );

    let part = self.insert(InsertDocumentPart {
      id,
      document_id: document_id.to_string(),
      friendly_name: friendly_name.to_string(),
      hash,
      content_type: content_type.to_string(),
      size_in_bytes,
      is_base
    }).save().await?;

    part.storage_put(payload).await?;

    Ok(part)
  }
}

impl DocumentPart {
  pub async fn add_signature(&self, signed_payload: &SignedPayload) -> sqlx::Result<DocumentPartSignature> {
    let serialized_signature = signed_payload.signature.serialize();

    self.state.document_part_signature().insert(
      InsertDocumentPartSignature{
        document_part_id: self.attrs.id.clone(),
        pubkey_id: signed_payload.signer.to_string(),
        signature: serialized_signature.to_vec(),
        signature_hash: hexdigest(&serialized_signature),
      }
    ).save_and_trigger_updates().await
  }

  pub async fn contents(&self) -> ConstataResult<Vec<u8>> {
    self.storage_fetch().await
  }
}

