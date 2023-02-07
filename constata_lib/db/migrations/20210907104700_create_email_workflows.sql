DROP TABLE fundings;

CREATE TYPE payment_method AS ENUM (
  'banktransferbbvaeur',
  'stripeeur',
  'paypaleur',
  'bitexbtc',
  'santaclaus'
);

CREATE TABLE fundings (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  quote_reference VARCHAR NOT NULL,
  quoted_amount DECIMAL NOT NULL,
  quoted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  payment_method payment_method NOT NULL,
  bulletin_id INTEGER NOT NULL DEFAULT current_draft(),
  tokens BIGINT NOT NULL,
  state VARCHAR NOT NULL,
  received_amount DECIMAL,
  fees DECIMAL,
  clearing_data BYTEA,
  received_at TIMESTAMPTZ
);

CREATE INDEX fundings_person_id ON fundings (person_id);

ALTER TABLE documents ADD cost BIGINT;
ALTER TABLE documents ADD funded BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE documents ADD funded_at TIMESTAMPTZ;
ALTER TABLE documents ADD created_at TIMESTAMPTZ NOT NULL DEFAULT now();

UPDATE documents d SET 
  funded_at = (SELECT b.started_at FROM bulletins b WHERE b.id = d.bulletin_id),
  created_at = (SELECT b.started_at FROM bulletins b WHERE b.id = d.bulletin_id),
  funded = true,
  cost = (SELECT ceil(dp.size_in_bytes / 1024) FROM document_parts dp WHERE dp.document_id = d.id AND dp.is_base )
;

ALTER TABLE documents ALTER COLUMN cost SET NOT NULL;
ALTER TABLE documents ALTER COLUMN bulletin_id DROP NOT NULL;
ALTER TABLE documents ALTER COLUMN bulletin_id DROP DEFAULT;
ALTER TABLE document_part_signatures ALTER COLUMN bulletin_id DROP NOT NULL;
ALTER TABLE document_part_signatures ALTER COLUMN bulletin_id DROP DEFAULT;

ALTER TABLE bulletins ADD block_time TIMESTAMPTZ;
