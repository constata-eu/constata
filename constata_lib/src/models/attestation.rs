use crate::{
  signed_payload::SignedPayload,
  models::{
    model,
    Person,
    Story,
    Org,
    OrgDeletion,
    UtcDateTime,
    Site,
    email_callback::*,
  },
  Result,
};

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
  ) -> Result<Attestation> {
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
            cc: true,
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

describe! {
  regtest!{ creates_an_attestation (site, c, _chain)
    let alice = c.alice().await;
    let payloads = vec![
      alice.signed_payload(b"hello world"),
      alice.signed_payload(b"goodbye world"),
    ];
    let att = site.attestation().create(&alice.person().await, &payloads, None, None, None).await?;
    assert_eq!(att.org_id(), alice.org().await.id());
  }
}
