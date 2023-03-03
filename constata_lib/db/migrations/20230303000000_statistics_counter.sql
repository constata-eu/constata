ALTER TABLE download_proof_links ADD COLUMN admin_visited BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE download_proof_links ADD COLUMN public_visit_count INTEGER NOT NULL DEFAULT 0;
