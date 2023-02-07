use crate::{
  models::{
    model,
    Site,
    pubkey::*,
    document_part::*,
  },
};

model!{
  state: Site,
  table: document_part_signatures,
  struct DocumentPartSignature {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    document_part_id: String,
    #[sqlx_model_hints(varchar)]
    pubkey_id: String,
    signature: Vec<u8>,
    #[sqlx_model_hints(varchar)]
    signature_hash: String,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: Option<i32>,
  },
  belongs_to {
    Pubkey(pubkey_id),
    DocumentPart(document_part_id),
  }
}

impl InsertDocumentPartSignatureHub {
  pub async fn save_and_trigger_updates(self) -> sqlx::Result<DocumentPartSignature> {
    let mut signature = self.save().await?;

    if signature.document_part().await?.document().await?.is_accepted() {
      signature = signature.accept().await?;
    }

    Ok(signature)
  }
}

impl DocumentPartSignature {
  pub async fn accept(self) -> sqlx::Result<DocumentPartSignature> {
    let (tx, draft) = self.state.bulletin().current_draft().await?;
    let signature = self.update().bulletin_id(Some(draft.into_inner().attrs.id)).save().await?;
    tx.commit().await?;
    Ok(signature)
  }
}

