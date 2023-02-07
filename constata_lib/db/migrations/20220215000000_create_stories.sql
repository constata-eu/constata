CREATE TABLE stories (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  open_until TIMESTAMPTZ,
  markers TEXT NOT NULL DEFAULT '',
  private_markers TEXT NOT NULL DEFAULT ''
);

CREATE INDEX stories_person_id ON stories (person_id);

CREATE TABLE story_snapshots (
  id SERIAL PRIMARY KEY NOT NULL,
  story_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  hash VARCHAR NOT NULL,
  bulletin_id INTEGER NOT NULL DEFAULT current_draft()
);

CREATE INDEX story_snapshots_story_id ON story_snapshots (story_id);
CREATE INDEX story_snapshots_hash ON story_snapshots (hash);
CREATE INDEX story_snapshots_bulletin_id ON story_snapshots (bulletin_id);

ALTER TABLE documents ADD story_id INTEGER;
ALTER TABLE documents ADD author_id INTEGER;

CREATE FUNCTION "create_stories"() RETURNS boolean
    LANGUAGE plpgsql
AS
$$
DECLARE
    document RECORD;
    part_hash varchar;
BEGIN
    FOR document IN SELECT id, person_id, created_at FROM documents LOOP
      INSERT INTO stories (person_id) VALUES (document.person_id);
      UPDATE documents SET story_id = currval('stories_id_seq'::regclass), author_id = document.person_id WHERE id = document.id;
      SELECT hash INTO part_hash FROM document_parts WHERE document_id = document.id AND is_base LIMIT 1;
      INSERT INTO story_snapshots (
        id,
        story_id,
        hash,
        "created_at",
        bulletin_id
      ) VALUES (
        currval('stories_id_seq'::regclass),
        currval('stories_id_seq'::regclass),
        encode(sha256(decode(CONCAT(
          currval('stories_id_seq'::regclass)::varchar,
          '-',
          document.person_id::varchar,
          '-null-',
          document.id
        ), 'escape')), 'hex'),
        document.created_at,
        current_draft()
      );
    END LOOP;

    RETURN TRUE;
END
$$;

select create_stories();

ALTER TABLE documents ALTER COLUMN story_id SET NOT NULL;
ALTER TABLE documents ALTER COLUMN author_id SET NOT NULL;

CREATE INDEX documents_story_id ON documents (story_id);
CREATE INDEX documents_person_author_id ON documents (author_id);

ALTER TABLE pubkey_domain_endorsements ADD person_id INTEGER;
UPDATE pubkey_domain_endorsements SET person_id = p.person_id FROM pubkeys p WHERE p.id = pubkey_id;
ALTER TABLE pubkey_domain_endorsements ALTER COLUMN person_id SET NOT NULL;
CREATE INDEX pubkey_domain_endorsement_person_id ON pubkey_domain_endorsements (person_id);

CREATE TABLE kyc_endorsements (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  story_id INTEGER NOT NULL,
  name VARCHAR,
  last_name VARCHAR,
  id_number VARCHAR,
  id_type VARCHAR,
  birthdate TIMESTAMPTZ,
  nationality VARCHAR,
  country VARCHAR,
  job_title VARCHAR,
  legal_entity_name VARCHAR,
  legal_entity_country VARCHAR,
  legal_entity_registration VARCHAR,
  legal_entity_tax_id VARCHAR
);

ALTER TABLE pubkey_domain_endorsements ALTER COLUMN bulletin_id SET DEFAULT NULL;

ALTER TABLE email_addresses ADD evidence_hash VARCHAR;
UPDATE pubkey_domain_endorsements SET bulletin_id = NULL WHERE state != 'accepted';
UPDATE pubkey_domain_endorsements SET bulletin_id = current_draft() WHERE state = 'accepted';

UPDATE email_addresses SET evidence_hash = encode(sha256(evidence), 'hex');
ALTER TABLE email_addresses ALTER COLUMN evidence_hash SET NOT NULL;

ALTER TABLE email_addresses ALTER COLUMN bulletin_id SET DEFAULT current_draft();
UPDATE email_addresses SET bulletin_id = current_draft();

