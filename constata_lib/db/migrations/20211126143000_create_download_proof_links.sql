CREATE TABLE download_proof_links (
  id SERIAL PRIMARY KEY NOT NULL,
  token VARCHAR NOT NULL,
  document_id VARCHAR NOT NULL,
  valid_until TIMESTAMPTZ NOT NULL
);

CREATE INDEX download_proof_links_token ON download_proof_links (token);
