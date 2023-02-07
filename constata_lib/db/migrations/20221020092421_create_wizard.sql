CREATE TYPE template_kind AS ENUM (
  'diploma',
  'attendance',
  'invitation'
);

ALTER TABLE certos_templates ADD COLUMN hidden BOOLEAN NOT NULL DEFAULT true;
UPDATE certos_templates SET hidden = false;
ALTER TABLE certos_templates ADD COLUMN kind template_kind NOT NULL DEFAULT 'diploma';
ALTER TABLE certos_templates ADD COLUMN schema VARCHAR;

ALTER TABLE email_addresses ADD COLUMN access_token_id INTEGER;
ALTER TABLE email_addresses ADD COLUMN verified_at TIMESTAMPTZ;
ALTER TABLE email_addresses ADD COLUMN keep_private BOOLEAN NOT NULL DEFAULT false;
UPDATE email_addresses SET verified_at = now();

CREATE TYPE outgoing_email_message_kind AS ENUM (
  'welcome',
  'kycrequestreceived',
  'emailverification'
);

CREATE TABLE outgoing_email_messages (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  email_address_id INTEGER NOT NULL,
  sent_at TIMESTAMPTZ,
  kind outgoing_email_message_kind NOT NULL
);
CREATE INDEX outgoing_email_messages_sent_at ON outgoing_email_messages (sent_at);
CREATE INDEX outgoing_email_messages_person_id ON outgoing_email_messages (person_id);
CREATE INDEX outgoing_email_messages_email_address_id ON outgoing_email_messages (email_address_id);

ALTER TABLE pubkeys ADD COLUMN encrypted_key VARCHAR;
ALTER TABLE pubkeys ADD COLUMN public_key VARCHAR;

ALTER TABLE kyc_endorsements ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
UPDATE kyc_endorsements SET updated_at = created_at;

CREATE TYPE access_token_kind AS ENUM (
  'verify_email',
  'accept_tyc'
);

CREATE TABLE access_tokens (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  org_id INTEGER NOT NULL,
  token VARCHAR NOT NULL,
  kind access_token_kind NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  auto_expires_on TIMESTAMPTZ NOT NULL DEFAULT now() + '30 days',
  expired BOOLEAN NOT NULL DEFAULT false
);
