CREATE TABLE bumps (
  id SERIAL PRIMARY KEY NOT NULL,
  bulletin_id INTEGER NOT NULL,
  counter INTEGER NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  raw_transaction TEXT NOT NULL,
  raw_transaction_hash VARCHAR NOT NULL
);

CREATE INDEX bumps_bulletin_id ON bumps (bulletin_id);
CREATE INDEX bumps_raw_transaction_hash ON bumps (raw_transaction_hash);

ALTER TABLE bulletins ADD COLUMN submitted_at TIMESTAMPTZ;