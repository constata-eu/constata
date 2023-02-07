/// A Public Key is a Bitcoin Address used for signing.
/// Although bitcoin addresses are not public key, a public key can be produced from a signed message.
/// A person can only have one public key.
/// This is a known limitation. Adding multiple keys will require a special design for the first key
/// and validation for subsequent keys.
/// The request to register a new public key must be signed by the corresponding private key.
/// We can't add public keys to the system without their constent.
use super::*;
use crate::{
  models::{person::*, hasher::hexdigest, Environment},
  signed_payload::SignedPayload,
  Base64Standard,
  Site,
  Error,
  Result
};
use bitcoin::Address;
use serde_with::serde_as;
use std::str::FromStr;
use juniper::GraphQLObject;

model!{
  state: Site,
  table: pubkeys,
  #[serde_as]
  struct Pubkey {
    #[sqlx_model_hints(varchar)]
    id: String,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[serde(with = "Base64Standard")]
    evidence_payload: Vec<u8>,
    evidence_signature: String,
    #[sqlx_model_hints(varchar)]
    hash: String,
    signature_hash: String,
    #[sqlx_model_hints(int4, default)]
    bulletin_id: i32,
    #[sqlx_model_hints(bigint, default)]
    nonce: i64,
    #[sqlx_model_hints(varchar)]
    encrypted_key: Option<String>,
    #[sqlx_model_hints(varchar)]
    public_key: Option<String>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Person(person_id),
    Org(org_id),
    OrgDeletion(deletion_id),
  }
}

#[derive(GraphQLObject, Serialize, Deserialize)]
#[graphql(description = "A credentials file")]
pub struct Credentials {
  pub address: String,
  pub public_key: String,
  pub encrypted_key: String,
  pub environment: Environment,
}

impl Pubkey {
  pub fn address(&self) -> Address {
    Address::from_str( self.id() )
      .expect(&format!("Database value was not an address {}", self.id()))
  }

  pub fn into_credentials(self) -> Option<Credentials> {
    let p = self.attrs;
    Some(Credentials {
      address: p.id,
      public_key: p.public_key?,
      encrypted_key: p.encrypted_key?,
      environment: self.state.settings.environment()
    })
  }
}

impl PubkeyHub {
  pub async fn validate_pubkey_creation(&self, person_id: &PersonId, signed_payload: &SignedPayload) -> Result<String> {
    let id = signed_payload.signer.to_string();

    if self.state.person().find_optional(person_id).await?.is_none() {
      return Err(Error::validation("person_id", "unknown_person_id"));
    }

    if self.state.pubkey().select().person_id_eq(person_id).optional().await?.is_some() {
      return Err(Error::validation("person_id", "person_can_only_have_one_pubkey"));
    }

    if self.state.pubkey().find_optional(&id).await?.is_some() {
      return Err(Error::validation("signed_payload/signer", "pubkey_exists_and_belongs_to_someone_else"));
    }

    if !signed_payload.signed_ok()? {
      return Err(Error::validation("signed_payload", "wrong_signature"));
    }

    Ok(id)
  }

  pub async fn create_from_signed_payload(&self,
    person_id: PersonId,
    signed_payload: &SignedPayload
  ) -> Result<Pubkey> {
    let id = self.validate_pubkey_creation(&person_id, signed_payload).await?;
    Ok(self.create(id, person_id, signed_payload, None, None).await?)
  }

  pub async fn create_from_signed_payload_with_backup(&self,
    person_id: PersonId,
    signed_payload: &SignedPayload,
    encrypted_key: &str,
    public_key: &str,
  ) -> Result<Pubkey> {
    let id = self.validate_pubkey_creation(&person_id, signed_payload).await?;
    Ok(self.create( id, person_id, signed_payload,
     Some(encrypted_key.to_string()), Some(public_key.to_string())
    ).await?)
  }

  pub async fn create_from_credentials_token(
    &self,
    email_credentials_token: CreateEmailCredentialsToken,
    signed_payload: &SignedPayload
  ) -> Result<Pubkey> {
    let person = email_credentials_token.person().await?;
    let id = self.validate_pubkey_creation(person.id(), signed_payload).await?;
    let email_address = person.last_email_address().await?;
    let expected_payload = format!("{}-{}", *email_address.address(), *email_credentials_token.id());

    if &signed_payload.payload != expected_payload.as_bytes() {
      return Err(Error::validation( "signed_payload/payload", "payload_should_be_email_address_plus_credentials_token"));
    }

    Ok(self.create(id, *person.id(), signed_payload, None, None).await?)
  }


  pub async fn create(&self, id: String, person_id: PersonId, signed_payload: &SignedPayload, encrypted_key: Option<String>, public_key: Option<String>) -> sqlx::Result<Pubkey> {
    let org_id = self.state.person().find(&person_id).await?.org().await?.attrs.id;
    self.insert(InsertPubkey{
      id,
      person_id,
      org_id,
      evidence_payload: signed_payload.payload.clone(),
      evidence_signature: signed_payload.signature.to_string(),
      hash: hexdigest(&signed_payload.signer.to_string().as_bytes()),
      signature_hash: hexdigest(&signed_payload.signature.serialize()),
      encrypted_key,
      public_key,
    }).save().await
  }
}

describe! {
  use std::str::FromStr;

  pub async fn make_pubkey(p: &TestDb) -> Result<Pubkey> {
    make_pubkey_with(&p, None, None).await
  }

  pub async fn make_pubkey_with(p: &TestDb, maybe_person_id: Option<PersonId>, maybe_payload: Option<&[u8]>) -> Result<Pubkey> {
    let person_id = if let Some(id) = maybe_person_id { id } else { p.make_person().await.attrs.id };
    let signed_payload = p.eve().await
      .signed_payload(maybe_payload.unwrap_or(b"Hello Constata.eu"));
    p.site.pubkey().create_from_signed_payload(person_id, &signed_payload).await
  }

  dbtest!{ creates_pubkey_for_person_from_form (site, c)
    let pubkey = site.pubkey().create_from_signed_payload(
      c.make_person().await.attrs.id,
      &c.eve().await.signed_payload(b"Hello Constata.eu"),
    ).await?;

    assert_that!(&pubkey.attrs, structure![ PubkeyAttrs {
      id: eq("mo1RHAgNG34rA3NpzGPfCGLKWKexWwcYAu".to_string()),
      bulletin_id: eq(1),
    }]);

    assert_eq!(pubkey.address(), Address::from_str("mo1RHAgNG34rA3NpzGPfCGLKWKexWwcYAu")?);
  }

  dbtest!{ fails_because_person_does_not_exist (_site, c)
    assert_that!(
      &make_pubkey_with(&c, Some(1), None).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("unknown_person_id".to_string()) } }
    );
  }

  dbtest!{ fails_because_person_already_had_a_public_key (_site, c)
    let result = make_pubkey_with(&c, Some(make_pubkey(&c).await?.attrs.person_id), None);
    assert_that!(
      &result.await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("person_can_only_have_one_pubkey".to_string()) } }
    );
  }

  dbtest!{ fails_because_public_key_belongs_to_someone_else (_site, c)
    make_pubkey(&c).await.unwrap();
    assert_that!(
      &make_pubkey_with(&c, Some(c.make_person().await.attrs.id), None).await.unwrap_err(),
      structure!{ Error::Validation{ message: eq("pubkey_exists_and_belongs_to_someone_else".to_string()) } }
    );
  }
}
