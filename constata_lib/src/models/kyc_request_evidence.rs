use super::*;
use crate::{
  models::Site ,
  Result,
};

model!{
  state: Site,
  table: kyc_request_evidences,
  struct KycRequestEvidence {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    kyc_request_id: i32,
    #[sqlx_model_hints(varchar)]
    filename: String,
    #[sqlx_model_hints(varchar)]
    content_type: String,
  },
  belongs_to{
    KycRequest(kyc_request_id),
  }
}

derive_storable!(KycRequestEvidence, "kre");

impl KycRequestEvidenceHub {
  pub async fn create(self, kyc_request: &KycRequest, filename: &str, payload: &[u8]) -> Result<KycRequestEvidence> {
    let kyc_request_evidence = self.insert(InsertKycRequestEvidence {
      kyc_request_id: kyc_request.attrs.id,
      filename: filename.to_string(),
      content_type: tree_magic_mini::from_u8(payload).to_string(),
    }).save().await?;
    kyc_request_evidence.storage_put(payload).await?;
    Ok(kyc_request_evidence)
  }
}

impl KycRequestEvidence {
  pub async fn contents(&self) -> Result<Vec<u8>> {
    self.storage_fetch().await
  }
}
