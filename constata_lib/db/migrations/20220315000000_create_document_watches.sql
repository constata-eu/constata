CREATE TABLE certos_apps (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL
);
CREATE INDEX certos_app_person_id ON certos_apps (person_id);

CREATE TABLE certos_templates (
  id SERIAL PRIMARY KEY NOT NULL,
  app_id INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  name VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  size_in_bytes INTEGER NOT NULL,
  payload BYTEA NOT NULL,
  custom_message TEXT
);
CREATE INDEX certos_templates_person_id ON certos_templates (person_id);
CREATE INDEX certos_templates_app_id ON certos_templates (app_id);

CREATE TABLE certos_requests (
  id SERIAL PRIMARY KEY NOT NULL,
  app_id INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  template_id INTEGER NOT NULL,
  state VARCHAR NOT NULL,
  name VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  size_in_bytes INTEGER NOT NULL,
  payload BYTEA NOT NULL 
);
CREATE INDEX certos_requests_person_id ON certos_requests (person_id);
CREATE INDEX certos_requests_app_id ON certos_requests (app_id);
CREATE INDEX certos_requests_state ON certos_requests (state);

CREATE TABLE certos_entries (
  id SERIAL PRIMARY KEY NOT NULL,
  app_id INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  request_id INTEGER NOT NULL,
  row_number INTEGER NOT NULL,
  state VARCHAR NOT NULL,
  size_in_bytes INTEGER NOT NULL,
  payload BYTEA NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  params TEXT NOT NULL,
  errors TEXT,
  document_id VARCHAR,
  email_callback_id INTEGER
);
CREATE INDEX certos_entries_person_id ON certos_entries (person_id);
CREATE INDEX certos_entries_request_id ON certos_entries (request_id);
CREATE INDEX certos_entries_app_id ON certos_entries (app_id);
CREATE INDEX certos_entries_state ON certos_entries (state);

ALTER TABLE email_callbacks ADD COLUMN custom_message TEXT;
