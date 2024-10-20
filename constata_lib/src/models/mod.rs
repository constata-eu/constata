use crate::prelude::*;
pub use sqlx::{self, types::Decimal};
use sqlx_models_orm::model;

pub_mods! {
  site;
  audit_log;
  blockchain;
  story;
  story_snapshot;
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
  pubkey;
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
  kyc_request_evidence;
  parked_reminder;
  outgoing_email_message;
  outgoing_email_message_kind;
  access_token;
  storable;
  template_schema;
  bump;
  storage;
  template;
  issuance;
  template_kind;
  wizard;
  attestation;
}

pub_mods! {
  story_bundle::StoryBundle;
  document_bundle::DocumentBundle;
}

pub mod document;
pub use document::{Document, DocumentHub, DocumentOrderBy, SelectDocument, SelectDocumentHub};
pub mod bulletin;
pub use bulletin::{Bulletin, BulletinHub, BulletinOrderBy, SelectBulletin, SelectBulletinHub};

pub mod entry;
pub use entry::{Entry, EntryHub, EntryOrderBy, InsertEntry, SelectEntry, SelectEntryHub};

pub mod web_callback;
pub use web_callback::{
  SelectWebCallback, SelectWebCallbackAttempt, SelectWebCallbackHub, WebCallback,
  WebCallbackAttempt, WebCallbackAttemptOrderBy, WebCallbackKind, WebCallbackOrderBy,
  WebCallbackResultCode, WebCallbackState,
};

pub mod pubkey_domain_endorsement;
pub use pubkey_domain_endorsement::{
  PubkeyDomainEndorsement, PubkeyDomainEndorsementOrderBy, SelectPubkeyDomainEndorsement,
  SelectPubkeyDomainEndorsementHub,
};

pub mod kyc_request;
pub use kyc_request::{
  InsertKycRequest, KycRequest, KycRequestOrderBy, KycRequestProcessForm, SelectKycRequest,
  SelectKycRequestHub,
};

pub mod endorsement;
pub use endorsement::{for_api as endorsement_for_api, Endorsement};

pub mod btcpay;
pub use mailers::email_parked_documents::*;
