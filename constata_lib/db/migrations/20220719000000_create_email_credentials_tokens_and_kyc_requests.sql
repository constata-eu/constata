CREATE TABLE create_email_credentials_tokens (
  id VARCHAR PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  evidence BYTEA
);

CREATE TABLE kyc_requests (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  state VARCHAR NOT NULL DEFAULT 'pending',
  name VARCHAR,
  last_name VARCHAR,
  id_number VARCHAR,
  id_type VARCHAR,
  birthdate TIMESTAMPTZ,
  nationality VARCHAR,
  country VARCHAR,
  job_title VARCHAR,
  legal_entity_name VARCHAR,
  legal_entity_country VARCHAR,
  legal_entity_registration VARCHAR,
  legal_entity_tax_id VARCHAR
);

CREATE TABLE kyc_request_evidences (
  id SERIAL PRIMARY KEY NOT NULL,
  kyc_request_id INTEGER NOT NULL,
  filename VARCHAR,
  content_type VARCHAR
);