use crate::prelude::*;
pub use sqlx::{self, types::Decimal};
use sqlx_models_orm::model;

pub_mods!{
  site;
  audit_log;
  blockchain;
  bulletin;
  story;
  story_snapshot;
  document;
  document_part;
  document_source;
  document_part_signature;
  email_address;
  email_callback;
  subscription;
  payment;
  payment_source;
  gift;
  invoice;
  account_state;
  hasher;
  org;
  person;
  proof;
  previewer;
  pubkey_domain_endorsement;
  pubkey;
  endorsement;
  mailers;
  magic_link;
  download_proof_link;
  invoice_link;
  admin_user;
  admin_user_session;
  terms_acceptance;
  kyc_endorsement;
  org_deletion;
  deletion_reason;
  kyc_request;
  kyc_request_evidence;
  parked_reminder;
  attestation;
  outgoing_email_message;
  outgoing_email_message_kind;
  access_token;
  storable;
  template_schema;
  web_callback;
  vc_prompt;
  vc_request;
  vc_requirement;
  bump;
  storage;
  template;
  issuance;
  entry;
  template_kind;
  wizard;
}

pub_mods!{
  vc_requirement_rules::VcRequirementRules;
  story_bundle::StoryBundle;
  document_bundle::DocumentBundle;
}

pub mod btcpay;
pub use mailers::email_parked_documents::*;
