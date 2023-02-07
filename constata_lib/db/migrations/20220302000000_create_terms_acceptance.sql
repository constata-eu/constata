CREATE TABLE terms_acceptances (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  token VARCHAR NOT NULL UNIQUE,
  accepted TIMESTAMPTZ,
  evidence BYTEA,
  bulletin_id INTEGER,
  hash VARCHAR
);

CREATE INDEX terms_acceptances_token ON terms_acceptances (token);
CREATE INDEX terms_acceptances_person_id ON terms_acceptances (person_id);

ALTER TABLE email_callbacks ADD COLUMN cc BOOLEAN NOT NULL DEFAULT FALSE;

