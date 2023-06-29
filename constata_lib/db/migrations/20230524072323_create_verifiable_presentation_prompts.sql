ALTER TYPE access_token_kind ADD VALUE 'vc_prompt';
ALTER TYPE access_token_kind ADD VALUE 'vc_request';

CREATE TYPE vc_request_state AS ENUM (
  'pending',
  'approved',
  'rejected',
  'failed'
);

CREATE TABLE vc_requirements (
  id SERIAL PRIMARY KEY NOT NULL,
  org_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  rules TEXT NOT NULL,
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deletion_id INTEGER
);

CREATE INDEX vc_requirements_org_id ON vc_requirements (org_id);

CREATE TABLE vc_prompts (
  id SERIAL PRIMARY KEY NOT NULL,
  org_id INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  name VARCHAR NOT NULL,
  access_token_id INTEGER NOT NULL,
  vc_requirement_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  deletion_id INTEGER
);

CREATE INDEX vc_prompts_org_id ON vc_prompts (org_id);
CREATE INDEX vc_prompts_archived_at ON vc_prompts (archived_at);
CREATE INDEX vc_prompts_access_token_id ON vc_prompts (access_token_id);

CREATE TABLE vc_requests (
  id SERIAL PRIMARY KEY NOT NULL,
  org_id INTEGER NOT NULL,
  vc_prompt_id INTEGER NOT NULL,
  access_token_id INTEGER NOT NULL,
  state vc_request_state DEFAULT 'pending',
  state_notes TEXT,
  vc_presentation TEXT,
  did TEXT,
  vidchain_code VARCHAR,
  vidchain_jwt VARCHAR,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ,
  deletion_id INTEGER
);

CREATE INDEX vc_requests_org_id ON vc_requests (org_id);
CREATE INDEX vc_requests_access_token_id ON vc_requests (access_token_id);
CREATE INDEX vc_requests_state ON vc_requests (state);
CREATE INDEX vc_requests_did ON vc_requests (did);
CREATE INDEX vc_requests_finished_at ON vc_requests (finished_at);

