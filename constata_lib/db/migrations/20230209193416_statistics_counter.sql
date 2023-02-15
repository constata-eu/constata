-- Add migration script here
ALTER TABLE download_proof_links ADD COLUMN admin_access_counter INTEGER DEFAULT 0;

ALTER TABLE download_proof_links ADD COLUMN public_access_counter INTEGER DEFAULT 0;