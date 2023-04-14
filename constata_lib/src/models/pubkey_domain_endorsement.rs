use super::*;
use crate::{
  models::{
    PersonId,
    hasher::hexdigest
  },
  signed_payload::SignedPayload,
  Error, Result, Site,
};
use bitcoin::{util::misc::MessageSignature, Address};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use duplicate::duplicate_item;

model!{
  state: Site,
  table: pubkey_domain_endorsements,
  struct PubkeyDomainEndorsement {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(varchar, default)]
    state: String,
    #[sqlx_model_hints(varchar)]
    pubkey_id: String,
    #[sqlx_model_hints(varchar)]
    domain: String,
    request_signature: String,
    #[sqlx_model_hints(int4, default)]
    attempts: i32,
    #[sqlx_model_hints(timestamptz, default)]
    next_attempt: UtcDateTime,
    #[sqlx_model_hints(varchar, default)]
    attempts_log: String,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: Option<i32>,
    #[sqlx_model_hints(varchar, default)]
    evidence: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    evidence_hash: Option<String>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    OrgDeletion(deletion_id),
  }
}

derive_storable!(PubkeyDomainEndorsement, "pkde");

impl PubkeyDomainEndorsementHub {
  pub async fn create_from_signed_payload(&self, signed_payload: &SignedPayload) -> Result<Pending> {
    if !signed_payload.signed_ok()? {
      return Err(Error::validation("signed_payload", "wrong_signature"));
    }

    let person = self.state.pubkey().find(&signed_payload.signer.to_string())
      .await
      .map_err(|_| Error::validation("signed_payload/signer", "signer_is_unknown"))?
      .person().await?;

    let domain_str = std::str::from_utf8(&signed_payload.payload)
      .map_err(|_| Error::validation("signed_payload/payload", "invalid_utf8_string"))?;

    let domain_url = url::Url::parse(domain_str)
      .map_err(|_| Error::validation("signed_payload/payload", "not_an_url"))?;

    if domain_url.scheme() != "https" && domain_url.host_str() != Some("127.0.0.1") {
      return Err(Error::validation("signed_payload/payload", "must_be_https"));
    }

    let pubkey_id = signed_payload.signer.to_string();

    let already_accepted_count = self.state.pubkey_domain_endorsement().select()
      .pubkey_id_eq(&pubkey_id)
      .domain_eq(&domain_str.to_string())
      .state_eq(&"accepted".to_string())
      .count().await?;

    if already_accepted_count > 0 {
      return Err(Error::validation("signed_payload/payload", "domain_already_verified"));
    }

    let pending_count = self.state.pubkey_domain_endorsement()
      .select()
      .state_eq(&"pending".to_string())
      .pubkey_id_eq(&pubkey_id)
      .count().await?;

    if pending_count > 0 {
      return Err(Error::validation( "signed_payload/payload", "cannot_have_more_than_one_endorsement_at_once"));
    }

    let object = self.insert(InsertPubkeyDomainEndorsement{
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      domain: domain_str.to_string(),
      pubkey_id: signed_payload.signer.to_string(),
      request_signature: signed_payload.signature.to_string()
    }).save().await?;

    object.in_pending()
  }
}

impl PubkeyDomainEndorsement {
  pub fn flow(&self) -> Flow {
    match self.state().as_ref() {
      "pending" => Flow::Pending(Pending(self.clone())),
      "accepted" => Flow::Accepted(Accepted(self.clone())),
      _ => Flow::Failed(Failed(self.clone())),
    }
  }

  pub async fn payload(&self) -> Result<Option<String>> {
    if self.evidence_hash().is_some() {
      let data = self.storage_fetch().await?;
      Ok(Some(String::from_utf8(data)?))
    } else {
      Ok(None)
    }
  }
}

#[duplicate_item(
  in_state        is_state        state_str       state_struct;
  [ in_pending  ] [ is_pending  ] [ "pending"   ] [ Pending   ];
  [ in_accepted ] [ is_accepted ] [ "accepted"  ] [ Accepted  ];
  [ in_failed   ] [ is_failed   ] [ "failed"    ] [ Failed    ];
)]
impl PubkeyDomainEndorsement {
  pub fn in_state(&self) -> Result<state_struct> {
    self.flow().in_state()
  }

  pub fn is_state(&self) -> bool {
    self.state().as_str() == state_str
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct PubkeyDomainEndorsementForm {
  pub signed_payload: SignedPayload,
}

impl PubkeyDomainEndorsementForm {
  pub async fn save(&self, site: &Site) -> Result<Pending> {
    site.pubkey_domain_endorsement().create_from_signed_payload(&self.signed_payload).await
  }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  // The endorsement was submitted and will be processed soon.
  Pending(Pending),
  // The endorsement fail due to retries.
  Failed(Failed),
  // The endorsement was processed. It's bulletin may not be processed yet though.
  Accepted(Accepted),
}

#[duplicate_item(flow_variant; [ Pending ]; [ Failed ]; [ Accepted ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(PubkeyDomainEndorsement);

#[duplicate_item(flow_variant; [ Pending ]; [ Failed ]; [ Accepted ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn pubkey_id(&self) -> &String { self.0.pubkey_id() }
  pub fn domain(&self) -> &String { self.0.domain() }
  pub fn request_signature(&self) -> &String { self.0.request_signature() }
  pub fn attempts(&self) -> &i32 { self.0.attempts() }
  pub fn attempts_log(&self) -> &String { self.0.attempts_log() }
  pub fn next_attempt(&self) -> &UtcDateTime { self.0.next_attempt() }
  pub fn into_inner(self) -> PubkeyDomainEndorsement { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a PubkeyDomainEndorsement { &self.0 }
}

#[duplicate_item(flow_variant; [ Failed ]; [ Accepted ];)]
impl flow_variant {
  pub fn bulletin_id(&self) -> &i32 { self.0.bulletin_id().as_ref().expect("Bulletin to be available here") }
  pub fn database_evidence(&self) -> &String { self.0.evidence().as_ref().expect("Evidence to be available here") }
  pub async fn evidence(&self) -> Result<String> { self.0.payload().await.map(|e| e.expect("Evidence to be available here") ) }
}

#[duplicate_item(
  in_state        is_state        variant(i)             state_struct;
  [ in_pending  ] [ is_pending  ] [ Flow::Pending(i)   ] [ Pending   ];
  [ in_accepted ] [ is_accepted ] [ Flow::Accepted(i)  ] [ Accepted  ];
  [ in_failed   ] [ is_failed   ] [ Flow::Failed(i)    ] [ Failed    ];
)]
impl Flow {
  pub fn in_state(&self) -> Result<state_struct> {
    if let variant([inner]) = self {
      Ok(inner.clone())
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a PubkeyDomainEndorsement {
    match self {
      Flow::Pending(a) => a.as_inner(),
      Flow::Accepted(a) => a.as_inner(),
      Flow::Failed(a) => a.as_inner(),
    }
  }
}

impl PubkeyDomainEndorsementHub {
  pub async fn process_all(&self) -> Result<Vec<Flow>> {
    let all = self.select()
      .state_eq(&"pending".to_string())
      .next_attempt_lte(&Utc::now())
      .all().await?;

    let mut endorsements = vec![];

    for item in all.into_iter() {
      endorsements.push(item.in_pending()?.process().await?);
    }

    Ok(endorsements)
  }
}

impl Pending {
  async fn process(self) -> Result<Flow> {
    match self.fetch_evidence().await {
      Ok(evidence) => {
        let (tx, draft) = self.as_inner().state.bulletin().current_draft().await?;
        let string = serde_json::to_string(&evidence)?;
        self.as_inner().storage_put(string.as_bytes()).await?;

        let flow = self.0.update()
          .state("accepted".to_string())
          .evidence(Some(string.clone()))
          .evidence_hash(Some(hexdigest(&string.as_bytes())))
          .bulletin_id(Some(*draft.id()))
          .save().await?
          .flow();

        tx.commit().await?;
        Ok(flow)
      },
      Err(error) => {
        let message = format!("{}{}\n", self.attempts_log(), error.to_string());
        let attempts = *self.attempts() + 1;
        let updater = self.0.update().attempts(attempts).attempts_log(message);

        if attempts >= 10 {
          Ok(updater.state("failed".to_string()).save().await?.flow())
        } else {
          let settings = Site::default().await?.settings;
          let next = Utc::now() + Duration::minutes(settings.pubkey_domain_endorsement_retry_wait);
          Ok(updater.next_attempt(next).save().await?.flow())
        }
      }
    }
  }

  async fn fetch_evidence(&self) -> Result<EndorsementEvidence> {
    use std::io::Read;
    let log = Site::default().await?.audit_log;
    let marker = log.start_marker();
    let response = ureq::get(&format!(
      "{}/constata_eu_domain_verification.txt",
      self.domain()
    ))
    .timeout(std::time::Duration::from_secs(3))
    .call();
    let logged = marker.extract();

    let mut body = String::new();
    response?
      .into_reader()
      .take(500)
      .read_to_string(&mut body)
      .map_err(|_| Error::validation("signature", "non-utf8 response body"))?;

    let signature = MessageSignature::from_str(body.trim())
      .map_err(|_| Error::validation("signature", "bad_signature_format"))?;

    let evidence = SignedPayload {
      signature,
      payload: self.domain().as_bytes().to_vec(),
      signer: Address::from_str(self.pubkey_id()).unwrap(),
    };

    if !evidence.signed_ok()? {
      return Err(Error::validation("identity", "wrong_signature"));
    }

    Ok(EndorsementEvidence { logged, body })
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct EndorsementEvidence {
  logged: String,
  body: String,
}

describe! {
  use crate::models::endorsement::Endorsement;

  dbtest!{ creates_an_endorsement_from_a_form (site, c)
    Site::default().await?.audit_log.start();
    let alice = c.alice().await;

    fn mock_response(response: &str) -> mockito::Mock {
      mockito::mock("GET", "/constata_eu_domain_verification.txt").with_status(200).with_body(response).create()
    }

    async fn assert_pending(site: &Site, log: &str, att: i32) {
      let pending = site.pubkey_domain_endorsement()
        .process_all().await
        .unwrap()[0].clone()
        .in_pending().unwrap();

      assert_eq!(pending.attempts(), &att);
      assert_that!(pending.attempts_log(), rematch(log));
    }

    let payload = alice.signed_payload(mockito::server_url().as_bytes());
    let form = PubkeyDomainEndorsementForm{ signed_payload: payload };
    let pending = form.clone().save(&site).await?.0.attrs;

    assert_that!(&pending, structure!{ PubkeyDomainEndorsementAttrs{
      id: eq(1),
      domain: rematch("http://127.0.0.1:1234"),
      pubkey_id: rematch("mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx"),
      request_signature: rematch("2XCewmmadlSQjPpA063UdzzsZL2yElZcqBcwrduQeOjxEiV2vSH"),
      attempts: eq(0),
    }});

    assert_pending(&site, "http://127.0.0.1:1234/constata_eu_domain_verification.txt: status code 501", 1).await;

    let _m = mock_response("asthoeuntsah");
    assert_pending(&site, "bad_signature_format", 2).await;

    let _m = mock_response("H3wYMefupoCH41NdTKAeJ2w6bw27iFV0glY7g5OriJxDNNpaobtQZ/rR+H6MgOXVcLhDu+Zkb0pINhpt69U4t+g=");
    assert_pending(&site, "wrong_signature", 3).await;

    let _m = mock_response("IP7/IUvwQ1+1yfr5UCcsoR5K+2XCewmmadlSQjPpA063UdzzsZL2yElZcqBcwrduQeOjxEiV2vSHPJ9icGf5f8E=\n\n");

    let accepted = site
      .pubkey_domain_endorsement()
      .process_all().await?[0]
      .in_accepted()?;

    assert_eq!(accepted.bulletin_id(), &1);
    assert_that!(accepted.database_evidence(), rematch("2XCewmmadlSQjPpA063UdzzsZL2"));
    assert_that!(&accepted.evidence().await?, rematch("2XCewmmadlSQjPpA063UdzzsZL2"));
    assert!(site.pubkey_domain_endorsement().process_all().await?.is_empty());

    assert_that!(
      &form.save(&site).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("domain_already_verified".to_string()) } }
    );

    assert_eq!(
      alice.person().await.endorsements().await?,
      vec![Endorsement::Website{ url: "http://127.0.0.1:1234".to_string() }]
    );
  }

  dbtest!{ fails_to_create_endorsement_form_with_invalid_url (site, c)
    let payload = c.alice().await.signed_payload(b"nada de nada");
    let form = PubkeyDomainEndorsementForm{ signed_payload: payload };
    assert_that!(
      &form.save(&site).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("not_an_url".to_string()) } }
    );
  }

  dbtest!{ fails_to_create_for_non_https (site, c)
    let payload = c.alice().await.signed_payload(b"http://example.com");
    let form = PubkeyDomainEndorsementForm{ signed_payload: payload };
    assert_that!(
      &form.save(&site).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("must_be_https".to_string()) } }
    );
  }

  dbtest!{ fails_to_create_for_non_utf8 (site, c)
    let payload = c.alice().await.signed_payload(&[33, 254, 0, 1]);
    let form = PubkeyDomainEndorsementForm{ signed_payload: payload };
    assert_that!(
      &form.save(&site).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("invalid_utf8_string".to_string()) } }
    );
  }

  dbtest!{ can_list_all_endorsements (site, c)
    let alice = c.alice().await;
    let alice_row = PubkeyDomainEndorsementForm{
      signed_payload: alice.signed_payload(b"https://example.com"),
    }.save(&site).await?;

    let bob_row = PubkeyDomainEndorsementForm{
      signed_payload: c.bob().await.signed_payload(b"https://example.com"),
    }.save(&site).await?;

    let list_pending = |filter| {
      let asite = site.clone();
      async move {
        asite.pubkey_domain_endorsement().select()
          .use_struct(SelectPubkeyDomainEndorsement{pubkey_id_eq: filter, ..Default::default() })
          .all().await.unwrap()
          .into_iter()
          .map(|x| x.in_pending().unwrap() )
          .collect::<Vec<Pending>>()
      }
    };

    assert_eq!(list_pending(None).await, vec![alice_row.clone(), bob_row.clone()]);
    assert_eq!(list_pending(Some(alice.public_key().to_string())).await, vec![alice_row]);
  }

  dbtest!{ fails_after_many_retries_but_can_be_resubmitted (site, c)
    let alice = c.alice().await;
    let form = PubkeyDomainEndorsementForm{
      signed_payload: alice.signed_payload(b"https://example.com"),
    };

    form.clone().save(&site).await?;

    // Cannot resubmit right away.
    assert_that!(
      &form.clone().save(&site).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("cannot_have_more_than_one_endorsement_at_once".to_string()) } }
    );

    for _ in 0..9 {
      site.pubkey_domain_endorsement().process_all().await?;
    }

    assert_eq!(10, *site.pubkey_domain_endorsement().process_all().await?[0].as_inner().attempts());

    assert_eq!(2, *form.clone().save(&site).await?.id());

    assert!(alice.person().await.endorsements().await?.is_empty())
  }

  dbtest!{ it_waits_between_retries (site, c)
    std::env::set_var("ROCKET_PUBKEY_DOMAIN_ENDORSEMENT_RETRY_WAIT", "10");
    let pending = PubkeyDomainEndorsementForm{
      signed_payload: c.alice().await.signed_payload(mockito::server_url().as_bytes()),
    }.save(&site).await?;
    let all = site.pubkey_domain_endorsement().process_all().await?;
    let processed = all[0].in_pending()?;
    assert_eq!(pending.id(), processed.id());
    assert!(*processed.next_attempt() > Utc::now());
    assert!(site.pubkey_domain_endorsement().process_all().await?.is_empty());
  }

  dbtest!{ fails_to_create_for_unknown_person (site, c)
    let form = PubkeyDomainEndorsementForm{
      signed_payload: c.eve().await.signed_payload(mockito::server_url().as_bytes()),
    };
    assert_that!(
      &form.save(&site).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("signer_is_unknown".to_string()) } }
    );
  }
}
