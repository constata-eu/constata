use crate::{models::hasher::hexdigest, Base64Standard, Result as MyResult};
use serde::{Deserialize, Serialize};
pub use rocket::{ http::Status, request::{FromRequest, Outcome, Request}};

use bitcoin::{
  consensus::{encode, Encodable},
  hashes::{sha256d, Hash, HashEngine},
  secp256k1,
  util::misc::{MessageSignature, BITCOIN_SIGNED_MSG_PREFIX},
  Address,
  PrivateKey,
};

use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct SignedPayload {
  #[serde(with = "Base64Standard")]
  pub payload: Vec<u8>,
  pub signer: Address,
  #[serde_as(as = "DisplayFromStr")]
  pub signature: MessageSignature,
}

impl SignedPayload {
  pub fn signed_msg_hash(msg: &[u8]) -> sha256d::Hash {
    let mut engine = sha256d::Hash::engine();
    engine.input(BITCOIN_SIGNED_MSG_PREFIX);
    let msg_len = encode::VarInt(msg.len() as u64);
    msg_len.consensus_encode(&mut engine).unwrap();
    engine.input(msg);
    sha256d::Hash::from_engine(engine)
  }

  pub fn payload_hash(&self) -> String {
    hexdigest(&self.payload)
  }

  pub fn unique_id(&self) -> String {
    hexdigest(format!("{}{}", &self.signer, &self.payload_hash()).as_bytes())
  }

  pub fn sign_with_key(message: &[u8], key: &PrivateKey) -> MessageSignature {
    let secp = secp256k1::Secp256k1::new();
    let msg_hash = SignedPayload::signed_msg_hash(message);
    let msg = secp256k1::Message::from_slice(&msg_hash).unwrap();
    let secp_sig = secp.sign_recoverable(&msg, &key.key);
    MessageSignature {
      signature: secp_sig,
      compressed: key.compressed,
    }
  }

  pub fn signed_ok(&self) -> MyResult<bool> {
    Ok(self.signature.is_signed_by_address(
      &secp256k1::Secp256k1::new(),
      &self.signer,
      SignedPayload::signed_msg_hash(&self.payload),
    )?)
  }

  pub fn pubkey(&self) -> MyResult<bitcoin::PublicKey> {
    Ok(self.signature.recover_pubkey(
      &secp256k1::Secp256k1::new(),
      SignedPayload::signed_msg_hash(&self.payload),
    )?)
  }
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for SignedPayload {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    async fn build(req: &Request<'_>) -> Option<SignedPayload> {
      let authorization = req.headers().get("Authorization").collect::<Vec<_>>().pop()?;
      let signed_payload: SignedPayload = serde_json::from_str(authorization).ok()?;
      
      Some(signed_payload)
    }

    match build(req).await {
      Some(session) => Outcome::Success(session),
      None => Outcome::Failure((Status::Unauthorized, ())),
    }
  }
}

describe! {
  test!{ deserializes_signed_payload_json
    let signed_payload: SignedPayload = serde_json::from_str(r#"{
      "payload":"aGVsbG8gd29ybGQ=",
      "signer":"mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx",
      "signature":"H6O6iC1NL18vjMVllny5oQz87Ir7O6n0v/rup8zBPjjAXWENMkJRcEQ69SRKXfw2QYen2PLt3amkY2bE+Fw623w="
    }"#).unwrap();
    assert!(signed_payload.signed_ok().unwrap());
    assert_that!(&signed_payload.payload, eq(b"hello world".to_vec()));
  }

  test!{ deserializes_bad_signatures_too
    let signed_payload: SignedPayload = serde_json::from_str(r#"{
      "payload":"bGVsbG8gd29ybGA=",
      "signer":"mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx",
      "signature":"H6O6iC1NL18vjMVllny5oQz87Ir7O6n0v/rup8zBPjjAXWENMkJRcEQ69SRKXfw2QYen2PLt3amkY2bE+Fw623w="
    }"#).unwrap();
    assert!(!signed_payload.signed_ok().unwrap());
  }
}
