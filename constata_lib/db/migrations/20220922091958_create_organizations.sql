CREATE TABLE orgs (
  id SERIAL PRIMARY KEY NOT NULL,
  subscription_id INTEGER,
  stripe_customer_id VARCHAR,
  public_name VARCHAR,
  logo_url VARCHAR,
  deletion_id INTEGER,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO orgs (id, public_name, logo_url, subscription_id, stripe_customer_id, deletion_id)
  SELECT id, nickname, logo_url, subscription_id, stripe_customer_id, person_deletion_id
  FROM persons;

ALTER TABLE persons ADD COLUMN org_id INTEGER;
ALTER TABLE persons ADD COLUMN suspended BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE persons ADD COLUMN admin BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE persons ADD COLUMN billing BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE terms_acceptances ADD COLUMN org_id INTEGER;
UPDATE terms_acceptances set org_id = person_id;
ALTER TABLE terms_acceptances ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX terms_acceptances_org_id ON terms_acceptances (org_id);

ALTER TABLE telegram_users ADD COLUMN org_id INTEGER;
UPDATE telegram_users set org_id = person_id;
ALTER TABLE telegram_users ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX telegram_users_org_id ON telegram_users (org_id);

ALTER TABLE telegram_bot_private_chats ADD COLUMN org_id INTEGER;
UPDATE telegram_bot_private_chats set org_id = person_id;
ALTER TABLE telegram_bot_private_chats ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX telegram_bot_private_chats_org_id ON telegram_bot_private_chats (org_id);

ALTER TABLE kyc_requests ADD COLUMN org_id INTEGER;
UPDATE kyc_requests set org_id = person_id;
ALTER TABLE kyc_requests ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX kyc_requests_org_id ON kyc_requests (org_id);

ALTER TABLE certos_requests ADD COLUMN org_id INTEGER;
UPDATE certos_requests set org_id = person_id;
ALTER TABLE certos_requests ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX certos_requests_org_id ON certos_requests (org_id);

ALTER TABLE certos_entries ADD COLUMN org_id INTEGER;
UPDATE certos_entries set org_id = person_id;
ALTER TABLE certos_entries ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX certos_entries_org_id ON certos_entries (org_id);

ALTER TABLE certos_templates ADD COLUMN org_id INTEGER;
UPDATE certos_templates set org_id = person_id;
ALTER TABLE certos_templates ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX certos_templates_org_id ON certos_templates (org_id);

ALTER TABLE pubkeys       ADD COLUMN org_id INTEGER;
UPDATE pubkeys SET org_id = person_id;
ALTER TABLE pubkeys ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX pubkeys_org_id ON pubkeys (org_id);

ALTER TABLE kyc_endorsements      ADD COLUMN org_id INTEGER;
UPDATE kyc_endorsements SET org_id = person_id;
ALTER TABLE kyc_endorsements ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX kyc_endorsements_org_id ON kyc_endorsements (org_id);

ALTER TABLE pubkey_domain_endorsements      ADD COLUMN org_id INTEGER;
UPDATE pubkey_domain_endorsements SET org_id = person_id;
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX pubkey_domain_endorsements_org_id ON pubkey_domain_endorsements (org_id);

ALTER TABLE email_addresses      ADD COLUMN org_id INTEGER;
UPDATE email_addresses SET org_id = person_id;
ALTER TABLE email_addresses ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX email_addresses_org_id ON email_addresses (org_id);

ALTER TABLE create_email_credentials_tokens  ADD COLUMN org_id INTEGER;
UPDATE create_email_credentials_tokens SET org_id = person_id;
ALTER TABLE create_email_credentials_tokens ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX create_email_credentials_tokens_org_id ON create_email_credentials_tokens (org_id);

ALTER TABLE stories      RENAME COLUMN person_id TO org_id;
ALTER INDEX stories_person_id RENAME TO stories_org_id;

ALTER TABLE documents      RENAME COLUMN person_id TO org_id;
ALTER INDEX documents_author_id RENAME TO documents_org_id;

ALTER TABLE documents      RENAME COLUMN author_id TO person_id;
ALTER INDEX documents_person_author_id RENAME TO documents_person_id;

ALTER TABLE subscriptions      RENAME COLUMN person_id TO org_id;
ALTER INDEX subscription_person_id RENAME TO subscriptions_org_id;

ALTER TABLE invoices           RENAME COLUMN person_id TO org_id;
ALTER INDEX invoices_person_id RENAME TO invoices_org_id;

ALTER TABLE payments           RENAME COLUMN person_id TO org_id;
ALTER INDEX payments_student_id RENAME TO payments_org_id;

ALTER TABLE invoice_links      RENAME COLUMN person_id TO org_id;

ALTER TABLE certos_apps        RENAME COLUMN person_id TO org_id;
ALTER INDEX certos_app_person_id RENAME TO certos_app_org_id;

ALTER TABLE gifts              RENAME COLUMN person_id TO org_id;
ALTER INDEX gift_person_id RENAME TO gifts_org_id;

ALTER TABLE parked_reminders   RENAME COLUMN person_id TO org_id;

ALTER TABLE person_deletions RENAME COLUMN person_id TO org_id;
ALTER TABLE person_deletions RENAME TO org_deletions;
ALTER INDEX person_deletions_person_id RENAME TO org_deletions_org_id;

/* Deletion columns */
ALTER TABLE create_email_credentials_tokens ADD COLUMN deletion_id INTEGER;
ALTER TABLE certos_entries RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE certos_requests RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE certos_templates RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE documents RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE download_proof_links RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE email_addresses RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE kyc_endorsements RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE persons RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE pubkeys RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE pubkey_domain_endorsements RENAME COLUMN person_deletion_id TO deletion_id;
ALTER TABLE stories RENAME COLUMN person_deletion_id TO deletion_id;

