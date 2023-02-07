CREATE TYPE deletion_reason AS ENUM (
  'userrequest',
  'constatadecision',
  'inactivity'
);

CREATE TABLE person_deletions (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  story_id INTEGER NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  reason deletion_reason NOT NULL,
  description VARCHAR NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  approving_admin_user INTEGER NOT NULL
);

CREATE INDEX person_deletions_person_id ON person_deletions (person_id);


ALTER TABLE email_addresses DROP CONSTRAINT email_addresses_pkey;
ALTER TABLE email_addresses RENAME COLUMN id TO address;
ALTER TABLE email_addresses ADD UNIQUE (address);
ALTER TABLE email_addresses ADD COLUMN id SERIAL PRIMARY KEY;

ALTER TABLE persons ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE documents ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE stories ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE pubkeys ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE kyc_endorsements ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE pubkey_domain_endorsements ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE email_addresses ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE download_proof_links ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE certos_requests ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE certos_templates ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;
ALTER TABLE certos_entries ADD COLUMN person_deletion_id INTEGER DEFAULT NULL;