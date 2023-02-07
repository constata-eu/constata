CREATE TABLE pubkey_domain_endorsements  (
  id SERIAL PRIMARY KEY NOT NULL,
  pubkey_id VARCHAR NOT NULL, -- Bitcoin address.
  state VARCHAR NOT NULL,
  domain VARCHAR NOT NULL,
  request_signature VARCHAR NOT NULL,
  attempts INTEGER NOT NULL,
  next_attempt TIMESTAMPTZ NOT NULL,
  attempts_log TEXT NOT NULL, 
  evidence TEXT,
  evidence_hash VARCHAR,
  bulletin_id INTEGER
);

CREATE TABLE document_part_signatures (
  id SERIAL PRIMARY KEY NOT NULL,
  document_part_id VARCHAR NOT NULL,
  pubkey_id VARCHAR NOT NULL, -- Bitcoin address.
  signature BYTEA NOT NULL,
  signature_hash VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL DEFAULT current_draft()
);

CREATE INDEX document_part_signatures_pubkey_id ON document_part_signatures (pubkey_id);
CREATE INDEX document_part_signatures_document_part_id ON document_part_signatures (document_part_id);

INSERT INTO document_part_signatures (pubkey_id, document_part_id, signature, signature_hash) SELECT
  sd.pubkey_id,
  dp.id,
  decode(sd.signature, 'base64'),
  encode(sha256(decode(sd.signature, 'base64')), 'hex')
  FROM signed_documents sd
    LEFT JOIN document_parts dp ON dp.document_id = sd.document_id AND dp.is_base
;
