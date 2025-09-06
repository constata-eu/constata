use super::*;
use crate::models::{PersonId, Site, UtcDateTime};

model! {
  state: Site,
  table: kyc_endorsements,
  struct KycEndorsement {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    story_id: PersonId,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    updated_at: UtcDateTime,
    #[sqlx_model_hints(varchar)]
    name: Option<String>,
    #[sqlx_model_hints(varchar)]
    last_name: Option<String>,
    #[sqlx_model_hints(varchar)]
    id_number: Option<String>,
    #[sqlx_model_hints(varchar)]
    id_type: Option<String>,
    #[sqlx_model_hints(timestamptz)]
    birthdate: Option<UtcDateTime>,
    #[sqlx_model_hints(varchar)]
    nationality: Option<String>,
    #[sqlx_model_hints(varchar)]
    country: Option<String>,
    #[sqlx_model_hints(varchar)]
    job_title: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_name: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_country: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_registration: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_tax_id: Option<String>,
    #[sqlx_model_hints(varchar)]
    legal_entity_linkedin_id: Option<String>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to{
    Person(person_id),
    Org(org_id),
    Story(story_id),
    OrgDeletion(deletion_id),
  },
}

/*
describe! {
  regtest!{ can_make_kyc_endorsement (site, c, _chain)
    let alice = c.alice().await;
    let person_id = alice.person_id.unwrap();
    let person = site.person().find(&person_id).await?;
    assert_that!(person.kyc_endorsement().await?.is_none());

    let kyc_endorsement = alice.make_kyc_endorsement().await;
    assert_that!(person.kyc_endorsement().await?.is_some());
    assert_that!(&kyc_endorsement.attrs, structure![KycEndorsementAttrs {
      name: maybe_some(rematch("Bruce")),
      last_name: maybe_some(rematch("Schneier")),
      job_title: maybe_some(rematch("CEO")),
      country: maybe_some(rematch("España")),
    }]);

    assert_eq!(kyc_endorsement.story().await?.documents().await?.len(), 1);
  }
}
*/
