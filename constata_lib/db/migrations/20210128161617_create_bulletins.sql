CREATE TABLE bulletins (
  id SERIAL PRIMARY KEY NOT NULL,
  state VARCHAR NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  hash VARCHAR,
  transaction TEXT, 
  transaction_hash VARCHAR,
  block_hash VARCHAR
);

CREATE INDEX bulletins_state ON bulletins (state);
CREATE INDEX bulletins_transaction_hash ON bulletins (transaction_hash);

CREATE function current_draft() RETURNS INTEGER LANGUAGE PLPGSQL AS $$
DECLARE
   draft_id integer;
BEGIN
  SELECT id INTO draft_id FROM bulletins WHERE state = 'draft' LIMIT 1;
  IF draft_id IS NULL THEN
    INSERT INTO bulletins (state, started_at) VALUES ('draft', now()) RETURNING id INTO draft_id;
  END IF;
  RETURN draft_id;
END;
$$;

CREATE TABLE persons (
  id SERIAL PRIMARY KEY NOT NULL
);

CREATE TABLE pubkeys (
  id VARCHAR PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  -- Evidence is just context captured by constata, there's always context.
  -- Evidence is a challenge, and the signature is a response.
  evidence_payload BYTEA NOT NULL,
  evidence_signature VARCHAR NOT NULL,
  hash VARCHAR NOT NULL,
  signature_hash VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL
);

CREATE INDEX pubkeys_bulletin_id ON pubkeys (bulletin_id);

CREATE TABLE email_addresses (
  id VARCHAR PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  -- Evidence is just context captured by constata, there's always context. This is hashed.
  evidence BYTEA NOT NULL,
  maybe_spoofed BOOLEAN NOT NULL DEFAULT FALSE,
  bulletin_id INTEGER NOT NULL
);

CREATE INDEX email_addresses_bulletin_id ON email_addresses (bulletin_id);

/* This is a biometric proof signed by the person. It is only valid if the person shown in the video is saying ABCDEF01234 */
CREATE TABLE biometric_proofs (
  id VARCHAR PRIMARY KEY NOT NULL,
  person_id VARCHAR NOT NULL,
  payload BYTEA NOT NULL,
  bulletin_id INTEGER NOT NULL
);
CREATE INDEX biometric_proofs_person_id ON biometric_proofs (person_id);
CREATE INDEX biometric_proofs_bulletin_id ON biometric_proofs (bulletin_id);

CREATE TABLE endorsements (
  id VARCHAR PRIMARY KEY NOT NULL,
  from_id VARCHAR NOT NULL,
  to_id VARCHAR NOT NULL,
  signature VARCHAR NOT NULL, -- This is signed by from_id
  bulletin_id INTEGER NOT NULL
);

CREATE INDEX endorsements_from_id ON endorsements (from_id);
CREATE INDEX endorsements_to_id ON endorsements (to_id);
CREATE INDEX endorsements_bulletin_id ON endorsements (bulletin_id);

CREATE TABLE witnessed_documents (
  document_id VARCHAR NOT NULL,
  evidence BYTEA NOT NULL
);

CREATE TABLE signed_documents (
  document_id VARCHAR NOT NULL,
  pubkey_id VARCHAR, -- Bitcoin address.
  signature VARCHAR -- Author signs the payload or payload hash.
);
CREATE INDEX signed_documents_pubkey_id ON signed_documents (pubkey_id);

CREATE TABLE documents (
  id VARCHAR PRIMARY KEY NOT NULL, -- A document ID concatenates author_id and the document hash.
  person_id INTEGER NOT NULL, -- Denormalized person id can be fetch trough pubkey too.
  indexer VARCHAR NOT NULL, -- This is an ENUM with all the available indexers.
  bulletin_id INTEGER NOT NULL
);

CREATE INDEX documents_author_id ON documents (person_id);
CREATE INDEX documents_bulletin_id ON documents (bulletin_id);

create TABLE document_parts (
  id VARCHAR PRIMARY KEY NOT NULL, -- Composed of document_id + position
  document_id VARCHAR NOT NULL,
  friendly_name VARCHAR NOT NULL,
  hash VARCHAR NOT NULL,
  content_type VARCHAR NOT NULL,
  size_in_bytes INTEGER NOT NULL,
  is_base BOOLEAN NOT NULL,
  payload BYTEA NOT NULL -- Plain contents of the part, we will always keep these private or offer the customer to not store them.
);

CREATE INDEX document_parts_document_id ON document_parts (document_id);
CREATE INDEX document_parts_hash ON document_parts (hash);

CREATE TABLE signature_requests (
  id VARCHAR PRIMARY KEY NOT NULL,
  document_id VARCHAR NOT NULL,
  signer_id VARCHAR NOT NULL,
  copy_id VARCHAR NOT NULL,
  signature VARCHAR NOT NULL, -- The document owner signs the request for this signature, we don't accept it otherwise.
  bulletin_id INTEGER NOT NULL
);

CREATE INDEX signature_requests_document_id ON signature_requests (document_id);
CREATE INDEX signature_requests_bulletin_id ON signature_requests (bulletin_id);

CREATE TABLE signatures (
  id VARCHAR PRIMARY KEY NOT NULL,
  request_id VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL,
  payload VARCHAR NOT NULL -- The actual signature from the recipient.
);

CREATE INDEX signatures_request_id ON signatures (request_id);
CREATE INDEX signatures_bulletin_id ON signatures (bulletin_id);

CREATE TABLE signature_acks (
  id VARCHAR PRIMARY KEY NOT NULL,
  signature_id VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL,
  payload VARCHAR NOT NULL -- The sender signs their ack on the recipient's signature.
);

CREATE INDEX signature_acks_signature_id ON signature_acks (signature_id);
CREATE INDEX signature_acks_bulletin_id ON signature_acks (bulletin_id);

CREATE TABLE approval_requests (
  id VARCHAR PRIMARY KEY NOT NULL,
  approver_id VARCHAR NOT NULL,
  document_id VARCHAR NOT NULL,
  signature VARCHAR NOT NULL, -- The person requesting the approval must sign.
  bulletin_id INTEGER NOT NULL
);
CREATE INDEX approval_requests_document_id ON approval_requests (document_id);
CREATE INDEX approval_requests_bulletin_id ON approval_requests (bulletin_id);

CREATE TABLE approvals (
  id VARCHAR PRIMARY KEY NOT NULL,
  request_id VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL,
  payload BYTEA NOT NULL, -- Evidence of this approval.
  signature VARCHAR NOT NULL -- Constata signature on the payload stating we have it.
);
CREATE INDEX approvals_request_id ON approvals (request_id);
CREATE INDEX approvals_bulletin_id ON approvals (bulletin_id);

CREATE TABLE approval_acks (
  id VARCHAR PRIMARY KEY NOT NULL,
  approval_id VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL,
  payload BYTEA NOT NULL --The sender accepts the presented evidence as enough.
);
CREATE INDEX approval_acks_approval_id ON approval_acks (approval_id);
CREATE INDEX approval_acks_bulletin_id ON approval_acks (bulletin_id);

CREATE TABLE fundings (
  id VARCHAR PRIMARY KEY NOT NULL,
  person_id VARCHAR NOT NULL,
  amount DECIMAL NOT NULL,
  payment_method VARCHAR NOT NULL,
  reference VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL,
  signature VARCHAR NOT NULL -- Constata signs this database entry. 
);

CREATE INDEX fundings_bulletin_id ON fundings (bulletin_id);
CREATE INDEX fundings_person_id ON fundings (person_id);

CREATE TABLE expenses (
  id VARCHAR PRIMARY KEY NOT NULL,
  person_id VARCHAR NOT NULL,
  amount DECIMAL NOT NULL,
  concept VARCHAR NOT NULL,
  entry_id VARCHAR NOT NULL,
  entry_type VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL,
  signature VARCHAR NOT NULL -- Constata signs this database entry. 
);

CREATE INDEX expenses_person_id ON expenses (person_id);
CREATE INDEX expenses_bulletin_id ON expenses (bulletin_id);
CREATE INDEX expenses_entry ON expenses (entry_type, entry_id);

