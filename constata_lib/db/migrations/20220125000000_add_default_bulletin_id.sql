ALTER TABLE pubkeys ALTER COLUMN bulletin_id SET DEFAULT current_draft();
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN bulletin_id SET DEFAULT current_draft();
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN state SET DEFAULT 'pending';
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN attempts SET DEFAULT 0;
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN next_attempt SET DEFAULT now();
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN attempts_log SET DEFAULT '';

ALTER TABLE witnessed_documents ADD COLUMN id SERIAL PRIMARY KEY;
