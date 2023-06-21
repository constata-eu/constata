use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "evidence of a kyc request")]
pub struct KycRequestEvidence {
  id: i32,
  filename: String,
  content_type: String,
  payload: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct KycRequestEvidenceFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  kyc_request_id_eq: Option<i32>,
}

#[rocket::async_trait]
impl Showable<db::KycRequestEvidence, KycRequestEvidenceFilter> for KycRequestEvidence {
  fn sort_field_to_order_by(field: &str) -> Option<KycRequestEvidenceOrderBy> {
    match field {
      "id" => Some(KycRequestEvidenceOrderBy::Id),
      _ => None,
    }
  }

  fn filter_to_select(f: KycRequestEvidenceFilter) -> SelectKycRequestEvidence {
    SelectKycRequestEvidence{
      id_in: f.ids,
      id_eq: f.id_eq,
      kyc_request_id_eq: f.kyc_request_id_eq,
      ..Default::default()
    }
  }
  
  async fn db_to_graphql(d: db::KycRequestEvidence) -> ConstataResult<Self> {
    let payload = d.contents().await?;
    Ok(KycRequestEvidence {
      id: d.attrs.id,
      filename: d.attrs.filename,
      content_type: d.attrs.content_type,
      payload: base64::encode(payload)
    })
  }
}
