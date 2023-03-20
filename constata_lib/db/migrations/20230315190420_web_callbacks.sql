CREATE TYPE web_callback_state AS ENUM (
  'pending',
  'done',
  'failed'
);

CREATE TYPE web_callback_kind AS ENUM (
  'attestation_done',
  'issuance_done',
  'token_purchase_required'
);

CREATE TYPE web_callback_result_code AS ENUM (
  'ok',
  'network_error',
  'non_success_response',
  'no_callbacks_url_for_org'
);

CREATE TABLE web_callbacks (
  id SERIAL PRIMARY KEY NOT NULL,
  org_id INTEGER NOT NULL,
  kind web_callback_kind NOT NULL,
  resource_id INTEGER NOT NULL,
  state web_callback_state NOT NULL DEFAULT 'pending',
  last_attempt_id INTEGER,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  next_attempt_on TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX web_callbacks_last_attempt_id ON web_callbacks (last_attempt_id);
CREATE INDEX web_callbacks_state ON web_callbacks (state);
CREATE INDEX web_callbacks_org_id ON web_callbacks (org_id);

CREATE TABLE web_callback_attempts (
  id SERIAL PRIMARY KEY NOT NULL,
  org_id INTEGER NOT NULL,
  web_callback_id INTEGER NOT NULL,
  attempted_at TIMESTAMPTZ NOT NULL,
  url VARCHAR NOT NULL,
  result_code web_callback_result_code NOT NULL,
  result_text VARCHAR NOT NULL
);
CREATE INDEX web_callback_attempts_web_callback_id ON web_callback_attempts (web_callback_id);

ALTER TABLE orgs ADD COLUMN web_callbacks_url VARCHAR;
